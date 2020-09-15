//#![nostd]

use rmodbus::server::context;
use std::fs::File;
use std::io::prelude::*;
use std::sync::MutexGuard;

fn looping() {
    loop {
        // READ WORK MODES ETC
        let mut ctx = context::CONTEXT.lock().unwrap();
        let _param1 = context::get(1000, &ctx.holdings).unwrap();
        let _param2 = context::get_f32(1100, &ctx.holdings).unwrap(); // ieee754 f32
        let _param3 = context::get_u32(1200, &ctx.holdings).unwrap(); // u32
        let cmd = context::get(1500, &ctx.holdings).unwrap();
        context::set(1500, 0, &mut ctx.holdings).unwrap();
        if cmd != 0 {
            println!("got command code {}", cmd);
            match cmd {
                1 => {
                    println!("saving memory context");
                    let _ = save("/tmp/plc1.dat", &mut ctx).map_err(|_| {
                        eprintln!("unable to save context!");
                    });
                }
                _ => println!("command not implemented"),
            }
        }
        drop(ctx);
        // ==============================================
        // DO SOME JOB
        // ..........
        // WRITE RESULTS
        let mut ctx = context::CONTEXT.lock().unwrap();
        context::set(0, true, &mut ctx.coils).unwrap();
        context::set_bulk(10, &(vec![10, 20]), &mut ctx.holdings).unwrap();
        context::set_f32(20, 935.77, &mut ctx.inputs).unwrap();
    }
}

fn save(fname: &str, ctx: &MutexGuard<context::ModbusContext>) -> Result<(), std::io::Error> {
    let mut file = match File::create(fname) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    for i in context::context_iter(&ctx) {
        match file.write(&[i]) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    }
    match file.sync_all() {
        Ok(_) => {}
        Err(e) => return Err(e),
    }
    return Ok(());
}

fn load(fname: &str, ctx: &mut MutexGuard<context::ModbusContext>) -> Result<(), std::io::Error> {
    let mut file = match File::open(fname) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut data: Vec<u8> = Vec::new();
    match file.read_to_end(&mut data) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }
    let mut writer = context::ModbusContextWriter::new(0);
    writer.write_bulk(data.as_slice(), ctx).unwrap();
    return Ok(());
}

#[path = "../../example-server/src/tcp.rs"]
mod tcp;

fn main() {
    // read context
    let unit_id = 1;
    {
        let mut ctx = context::CONTEXT.lock().unwrap();
        let _ = load(&"/tmp/plc1.dat", &mut ctx).map_err(|_| {
            eprintln!("warning: no saved context");
        });
    }
    use std::thread;
    thread::spawn(move || {
        tcp::tcpserver(unit_id, "localhost:5502");
    });
    looping();
}
