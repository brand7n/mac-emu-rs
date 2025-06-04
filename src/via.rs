/*  Rust translation of C code for minimal VIA emulation from umac
 *
 * Bare minimum support for ports A/B, shift register, and IRQs.
 * A couple of Mac-specific assumptions in here, as per comments...
 *
 * Copyright 2024 Matt Evans
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation files
 * (the "Software"), to deal in the Software without restriction,
 * including without limitation the rights to use, copy, modify, merge,
 * publish, distribute, sublicense, and/or sell copies of the Software,
 * and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 * BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use std::sync::Mutex;

lazy_static::lazy_static! {
    pub static ref VIA: Mutex<Option<Via>> = Mutex::new(None);
}

pub fn set_via(via: Via) {
    let mut lock = VIA.lock().unwrap();
    *lock = Some(via);
}

#[derive(Clone)]
pub struct ViaCallbacks {
    pub ra_change: Option<fn(u8)>,
    pub rb_change: Option<fn(u8)>,
    pub ra_in: Option<fn() -> u8>,
    pub rb_in: Option<fn() -> u8>,
    pub sr_tx: Option<fn(u8)>,
    pub irq_set: fn(bool),
}

pub struct Via {
    regs: [u8; 16],
    callbacks: ViaCallbacks,
    irq_active: u8,
    irq_enable: u8,
    irq_status: bool,
    sr_tx_pending: Option<u8>,
}

// VIA register indices
const VIA_RB: usize = 0;
const VIA_RA: usize = 1;
const VIA_DDRB: usize = 2;
const VIA_DDRA: usize = 3;
const VIA_SR: usize = 10;
const VIA_ACR: usize = 11;
const VIA_IFR: usize = 13;
const VIA_IRQ_CA: u8 = 0x01;
const VIA_IRQ_CB: u8 = 0x02;
const VIA_IRQ_SR: u8 = 0x04;
const VIA_IER: usize = 14;
const VIA_RA_ALT: usize = 15;

impl Via {
    pub fn new(callbacks: ViaCallbacks) -> Self {
        let mut regs = [0; 16];
        regs[VIA_RA] = 0x10; // Overlay, FIXME
        Self {
            regs,
            callbacks,
            irq_active: 0,
            irq_enable: 0,
            irq_status: false,
            sr_tx_pending: None,
        }
    }

    fn update_rega(&mut self, data: u8) {
        if self.regs[VIA_RA] != data {
            if let Some(f) = self.callbacks.ra_change {
                f(data);
            }
        }
    }

    fn update_regb(&mut self, data: u8) {
        if self.regs[VIA_RB] != data {
            if let Some(f) = self.callbacks.rb_change {
                f(data);
            }
        }
    }

    fn update_sr(&mut self, data: u8) {
        match self.regs[VIA_ACR] & 0x1c {
            0x1c => {
                if self.sr_tx_pending.is_some() {
                    // log: SR send while pending
                }
                self.sr_tx_pending = Some(data);
                self.irq_active |= VIA_IRQ_SR;
            }
            0x18 => {
                self.regs[VIA_SR] = 0;
            }
            _ => {}
        }
    }

    fn sr_done(&mut self) {
        if let Some(data) = self.sr_tx_pending.take() {
            if let Some(f) = self.callbacks.sr_tx {
                f(data);
            }
        }
    }

    fn assess_irq(&mut self) {
        let active = self.irq_enable & self.irq_active & 0x7f;
        let irq = active != 0;
        if irq != self.irq_status {
            (self.callbacks.irq_set)(irq);
            self.irq_status = irq;
        }
    }

    pub fn write(&mut self, addr: u32, data: u8) {
        let mut r = ((addr >> 9) & 0xf) as usize;
        let mut dowrite = true;
        match r {
            VIA_RA | VIA_RA_ALT => {
                self.update_rega(data);
                r = VIA_RA;
            }
            VIA_RB => self.update_regb(data),
            VIA_DDRA | VIA_DDRB => {} // FIXME
            VIA_SR => {
                self.update_sr(data);
                dowrite = false;
            }
            VIA_IER => {
                if data & 0x80 != 0 {
                    self.irq_enable |= data & 0x7f;
                } else {
                    self.irq_enable &= !(data & 0x7f);
                }
            }
            VIA_IFR => {
                let acked = self.irq_active & data;
                self.irq_active &= !data;
                if acked & VIA_IRQ_SR != 0 {
                    self.sr_done();
                }
            }
            _ => {}
        }
        if dowrite {
            self.regs[r] = data;
        }
        self.assess_irq();
    }

    fn read_ifr(&self) -> u8 {
        let active = self.irq_enable & self.irq_active & 0x7f;
        self.irq_active | if active != 0 { 0x80 } else { 0 }
    }

    fn read_reg(&self, reg: usize) -> u8 {
        match reg {
            VIA_RA | VIA_RA_ALT => {
                let input = self.callbacks.ra_in.map_or(0, |f| f());
                let ddr = self.regs[VIA_DDRA];
                (ddr & self.regs[VIA_RA]) | (!ddr & input)
            }
            VIA_RB => {
                let input = self.callbacks.rb_in.map_or(0, |f| f());
                let ddr = self.regs[VIA_DDRB];
                (ddr & self.regs[VIA_RB]) | (!ddr & input)
            }
            VIA_SR => {
                self.irq_active & !VIA_IRQ_SR;
                self.regs[VIA_SR]
            }
            VIA_IER => 0x80 | self.irq_enable,
            VIA_IFR => self.read_ifr(),
            _ => self.regs[reg],
        }
    }

    pub fn read(&mut self, addr: u32) -> u8 {
        let reg = ((addr >> 9) & 0xf) as usize;
        let val = self.read_reg(reg);
        self.assess_irq();
        val
    }

    pub fn tick(&mut self, _time_us: u64) {
        // FIXME: timer support
    }

    pub fn ca_event(&mut self, ca: u8) {
        match ca {
            1 => self.irq_active |= VIA_IRQ_CA,
            2 => self.irq_active |= VIA_IRQ_CB,
            _ => {}
        }
        self.assess_irq();
    }

    pub fn sr_rx(&mut self, val: u8) {
        if (self.regs[VIA_ACR] & 0x1c) == 0x0c {
            self.regs[VIA_SR] = val;
            self.irq_active |= VIA_IRQ_SR;
            self.assess_irq();
        }
    }
}