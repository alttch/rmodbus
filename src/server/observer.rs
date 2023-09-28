use super::context::MutContext;
use crate::server::context::Context;
use crate::{ErrorKind, VectorTrait};

struct ContextObserver<Ctx, FnPreWrite, FnPostWrite>
where
    Ctx: MutContext,
    FnPreWrite: FnMut(WriteEvent, &Ctx),
    FnPostWrite: FnMut(WriteEvent, &Ctx),
{
    pub ctx: Ctx,
    pub pre_write: Option<FnPreWrite>,
    pub post_write: Option<FnPostWrite>,
}

impl<Ctx, FnPreWrite, FnPostWrite> ContextObserver<Ctx, FnPreWrite, FnPostWrite>
where
    Ctx: MutContext,
    FnPreWrite: FnMut(WriteEvent, &Ctx),
    FnPostWrite: FnMut(WriteEvent, &Ctx),
{
    fn call<F>(&mut self, event: WriteEvent, f: F) -> Result<(), ErrorKind>
    where
        F: FnOnce(&mut Ctx) -> Result<(), ErrorKind>,
    {
        self.pre(event);
        let res = f(&mut self.ctx);
        self.post(event);

        res
    }

    fn pre(&mut self, event: WriteEvent) {
        if let Some(pre) = &mut self.pre_write {
            pre(event, &self.ctx)
        }
    }

    fn post(&mut self, event: WriteEvent) {
        if let Some(post) = &mut self.post_write {
            post(event, &self.ctx)
        }
    }
}

impl<Ctx, FnPreWrite, FnPostWrite> Context for ContextObserver<Ctx, FnPreWrite, FnPostWrite>
where
    Ctx: MutContext,
    FnPreWrite: FnMut(WriteEvent, &Ctx),
    FnPostWrite: FnMut(WriteEvent, &Ctx),
{
    fn get_coils_as_u8(
        &self,
        reg: u16,
        count: u16,
        buf: &mut impl VectorTrait<u8>,
    ) -> Result<(), ErrorKind> {
        self.ctx.get_coils_as_u8(reg, count, buf)
    }

    fn get_discretes_as_u8(
        &self,
        reg: u16,
        count: u16,
        buf: &mut impl VectorTrait<u8>,
    ) -> Result<(), ErrorKind> {
        self.ctx.get_discretes_as_u8(reg, count, buf)
    }

    fn get_inputs_as_u8(
        &self,
        reg: u16,
        count: u16,
        buf: &mut impl VectorTrait<u8>,
    ) -> Result<(), ErrorKind> {
        self.ctx.get_inputs_as_u8(reg, count, buf)
    }

    fn get_holdings_as_u8(
        &self,
        reg: u16,
        count: u16,
        buf: &mut impl VectorTrait<u8>,
    ) -> Result<(), ErrorKind> {
        self.ctx.get_holdings_as_u8(reg, count, buf)
    }
}

impl<Ctx, FnPreWrite, FnPostWrite> MutContext for ContextObserver<Ctx, FnPreWrite, FnPostWrite>
where
    Ctx: MutContext,
    FnPreWrite: FnMut(WriteEvent, &Ctx),
    FnPostWrite: FnMut(WriteEvent, &Ctx),
{
    fn set_coil(&mut self, reg: u16, val: bool) -> Result<(), ErrorKind> {
        let event = WriteEvent::Coils { reg, count: 1 };
        self.call(event, |ctx| ctx.set_coil(reg, val))
    }

    fn set_coils_from_u8(&mut self, reg: u16, count: u16, buf: &[u8]) -> Result<(), ErrorKind> {
        let event = WriteEvent::Coils { reg, count };
        self.call(event, |ctx| ctx.set_coils_from_u8(reg, count, buf))
    }

    fn set_holding(&mut self, reg: u16, val: u16) -> Result<(), ErrorKind> {
        let event = WriteEvent::Holdings { reg, count: 1 };
        self.call(event, |ctx| ctx.set_holding(reg, val))
    }

    fn set_holdings_from_u8(&mut self, reg: u16, buf: &[u8]) -> Result<(), ErrorKind> {
        let event = WriteEvent::Holdings {
            reg,
            count: buf.len() as u16 / 2,
        };
        self.call(event, |ctx| ctx.set_holdings_from_u8(reg, buf))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WriteEvent {
    Coils { reg: u16, count: u16 },
    Holdings { reg: u16, count: u16 },
}
