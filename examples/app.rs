use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use rmodbus::server::context::ModbusContext;
use rmodbus::server::storage::ModbusStorageFull;

#[path = "servers/tcp.rs"]
mod srv;

// put 1 to holding register 1500 to save current context to /tmp/plc1.dat
// if the file exists, context will be loaded at the next start

fn looping() {
    println!("Loop started");
    loop {
        // READ WORK MODES ETC
        let ctx = srv::CONTEXT.read().unwrap();
        let _param1 = ctx.get_holding(1000).unwrap();
        let _param2 = ctx.get_holdings_as_f32(1100).unwrap(); // ieee754 f32
        let _param3 = ctx.get_holdings_as_u32(1200).unwrap(); // u32
        let cmd = ctx.get_holding(1500).unwrap();
        drop(ctx);
        if cmd != 0 {
            println!("got command code {}", cmd);
            let mut ctx = srv::CONTEXT.write().unwrap();
            ctx.set_holding(1500, 0).unwrap();
            match cmd {
                1 => {
                    println!("saving memory context");
                    let _ = save("/tmp/plc1.dat", &ctx).map_err(|_| {
                        eprintln!("unable to save context!");
                    });
                }
                _ => println!("command not implemented"),
            }
        }
        // ==============================================
        // DO SOME JOB
        // ..........
        // WRITE RESULTS
        let mut ctx = srv::CONTEXT.write().unwrap();
        ctx.set_coil(0, true).unwrap();
        ctx.set_holdings_bulk(10, &[10, 20]).unwrap();
        ctx.set_inputs_from_f32(20, 935.77).unwrap();
    }
}

fn save(fname: &str, ctx: &ModbusStorageFull) -> Result<(), Box<dyn Error>> {
    let config = bincode::config::standard();
    let mut file = File::create(fname)?;
    file.write_all(&bincode::encode_to_vec(ctx, config)?)?;
    file.sync_all()?;
    Ok(())
}

fn load(fname: &str, ctx: &mut ModbusStorageFull) -> Result<(), Box<dyn Error>> {
    let config = bincode::config::standard();
    let mut file = File::open(fname)?;
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;
    let (bctx, _): (Box<ModbusStorageFull>, usize) = bincode::decode_from_slice(&data, config)?;
    *ctx = *bctx;
    Ok(())
}

fn main() {
    // read context
    let unit_id = 1;
    {
        let mut ctx = srv::CONTEXT.write().unwrap();
        let _ = load("/tmp/plc1.dat", &mut ctx).map_err(|_| {
            eprintln!("warning: no saved context");
        });
    }
    std::thread::spawn(move || {
        srv::tcpserver(unit_id, "localhost:5502");
    });
    looping();
}
