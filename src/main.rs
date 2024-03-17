use std::io::Write;
use std::thread::spawn;

use anyhow::Context;
use clap::Parser;
use rhai::{AST, Engine, Scope};

/// Run Rhai scripts against lines of stdin.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    /// Code to run before evaluating input
    #[arg(short, long)]
    before: Option<String>,

    /// Code to run after evaluating input
    #[arg(short, long)]
    after: Option<String>,

    /// The ryk program to run (in Rhai script)
    /// The program is run on each line in the input (read from stdin) and is provided to the
    /// program in the variable name 'line'.
    #[arg()]
    program: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // TODO constantize this and/or determine an optimal size
    // For now, I am assuming the thread reading stdin is always going to be faster
    // than the processing thread running rhai so a small buffer should be ok.
    let (tx, rx) = std::sync::mpsc::sync_channel(64);
    let (stdout_tx, stdout_rx) = std::sync::mpsc::channel();

    let mut engine = Engine::new();
    engine.register_fn("p", move |x: rhai::Dynamic| { stdout_tx.send(x).unwrap(); });
    let engine = engine;

    let before_ast = if let Some(b) = args.before {
        Some(engine.compile(b)?)
    } else {
        None
    };

    let after_ast = if let Some(a) = args.after {
        Some(engine.compile(a)?)
    } else {
        None
    };

    let ast = engine.compile(&args.program)?;

    let processing_thread = spawn(move || {
        process(engine, before_ast, ast, after_ast, rx)
    });

    let reading_thread = spawn(move || { read(tx) });

    let output_thread = spawn(move || { write(stdout_rx) });

    reading_thread.join().expect("reading thread should not panic")?;
    processing_thread.join().expect("processing thread should not panic")?;
    output_thread.join().expect("output thread should not panic")?;

    Ok(())
}

fn read(sender: std::sync::mpsc::SyncSender<String>) -> anyhow::Result<()> {
    // TODO It'd be nice to explore a way to not allocate each line (can I pass a &str into rhai?)
    // I think doing so (or using a pool of String) would require a return channel in order to know
    // when the other end was done using the string.
    for l in std::io::stdin().lines() {
        sender.send(l?)?
    }
    Ok(())
}

fn write(rx: std::sync::mpsc::Receiver<rhai::Dynamic>) -> anyhow::Result<()> {
    let mut stdout = std::io::stdout().lock();
    while let Ok(line) = rx.recv() {
        writeln!(&mut stdout, "{line}")?;
    }

    Ok(())
}

fn process(engine: Engine, before_ast: Option<AST>, ast: AST, after_ast: Option<AST>, line_rx: std::sync::mpsc::Receiver<String>) -> anyhow::Result<()> {
    // initialize a scope in order to save state across runs
    let mut scope = Scope::new();

    if let Some(b) = before_ast {
        engine.run_ast_with_scope(&mut scope, &b)?;
    }

    while let Ok(line) = line_rx.recv() {
        scope.set_or_push("line", line);
        engine.run_ast_with_scope(&mut scope, &ast)
            .context("running provided ryk program")?;
    }

    if let Some(a) = after_ast {
        engine.run_ast_with_scope(&mut scope, &a)?;
    }

    Ok(())
}
