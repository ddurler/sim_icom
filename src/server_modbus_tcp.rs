//! Serveur TCP pour les requêtes MODBUS/TCP dans la [`Database`]

//Le code ci-dessous est très largement inspiré de
//(ce dépôt)[https://github.com/slowtec/tokio-modbus/blob/main/examples/tcp-server.rs]

use std::sync::{Arc, Mutex};

use futures::future;

use tokio_modbus::prelude::*;

use crate::database::{Database, IdUser};

/// Adresse MODBUS max: Sans effet pour toutes les actions après cette adresse mots
pub const MODBUS_TOP_WORD_ADDRESS: u16 = 0x8000;

/// Wrapper de [`Database`] pour le serveur MODBUS/TCP
pub struct DatabaseService {
    thread_db: Arc<Mutex<Database>>,
    id_user: IdUser,
    debug_level: u8,
}

impl DatabaseService {
    /// Constructeur
    pub fn new(thread_db: Arc<Mutex<Database>>, id_user: IdUser, debug_level: u8) -> Self {
        Self {
            thread_db,
            id_user,
            debug_level,
        }
    }
}

impl tokio_modbus::server::Service for DatabaseService {
    type Request = Request<'static>;
    type Response = Response;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            Request::ReadInputRegisters(addr, cnt) => {
                let values = register_read(
                    &self.thread_db.lock().unwrap(),
                    self.id_user,
                    self.debug_level,
                    addr,
                    cnt,
                );
                future::ready(Ok(Response::ReadInputRegisters(values)))
            }
            Request::ReadHoldingRegisters(addr, cnt) => {
                let values = register_read(
                    &self.thread_db.lock().unwrap(),
                    self.id_user,
                    self.debug_level,
                    addr,
                    cnt,
                );
                future::ready(Ok(Response::ReadHoldingRegisters(values)))
            }
            Request::WriteMultipleRegisters(addr, values) => {
                register_write(
                    &mut self.thread_db.lock().unwrap(),
                    self.id_user,
                    self.debug_level,
                    addr,
                    &values,
                );
                #[allow(clippy::cast_possible_truncation)]
                future::ready(Ok(Response::WriteMultipleRegisters(
                    addr,
                    values.len() as u16,
                )))
            }
            Request::WriteSingleRegister(addr, value) => {
                register_write(
                    &mut self.thread_db.lock().unwrap(),
                    self.id_user,
                    self.debug_level,
                    addr,
                    std::slice::from_ref(&value),
                );
                future::ready(Ok(Response::WriteSingleRegister(addr, value)))
            }
            _ => {
                eprintln!("Server MODBUS/TCP: Unimplemented function code in request: {req:?} !!!");
                future::ready(Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Unimplemented function code in request".to_string(),
                )))
            }
        }
    }
}

/// Helper function implementing reading registers from [`Database`].
/// Used by both the input registers reading and the holding registers reading
fn register_read(db: &Database, id_user: IdUser, debug_level: u8, addr: u16, cnt: u16) -> Vec<u16> {
    let mut response_values = vec![0; cnt.into()];
    for i in 0..cnt {
        let reg_addr = addr + i;
        if reg_addr < MODBUS_TOP_WORD_ADDRESS {
            response_values[i as usize] = db.get_u16_from_word_address(id_user, reg_addr);
        } else {
            eprintln!("Server MODBUS/TCP: Read out of database {addr:04X} !!!");
        }
    }
    if debug_level > 1 {
        println!("Server MODBUS/TCP: Read {cnt} words @{addr:04X}: {response_values:?}");
    }
    response_values
}

/// Write a holding register. Used by both the write single register
/// and write multiple registers requests.
fn register_write(db: &mut Database, id_user: IdUser, debug_level: u8, addr: u16, values: &[u16]) {
    if debug_level > 1 {
        println!(
            "Server MODBUS/TCP: Write {} words @{:04X}: {:?}",
            values.len(),
            addr,
            values
        );
    }
    for (i, value) in values.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let reg_addr = addr + i as u16;
        if reg_addr < MODBUS_TOP_WORD_ADDRESS {
            db.set_u16_to_word_address(id_user, reg_addr, *value);
        } else {
            eprintln!("Server MODBUS/TCP: Write out of database {reg_addr:04X} !!!");
        }
    }
}
