//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use crate::{RCodec, WCodec, Zenoh080, Zenoh080Condition, Zenoh080Header};
use zenoh_buffers::{
    reader::{DidntRead, Reader},
    writer::{DidntWrite, Writer},
};
use zenoh_protocol::{
    common::imsg,
    core::WireExpr,
    zenoh::{zmsg, Pull},
};

impl<W> WCodec<&Pull, &mut W> for Zenoh080
where
    W: Writer,
{
    type Output = Result<(), DidntWrite>;

    fn write(self, writer: &mut W, x: &Pull) -> Self::Output {
        // Header
        let mut header = zmsg::id::PULL;
        if x.is_final {
            header |= zmsg::flag::F;
        }
        if x.max_samples.is_some() {
            header |= zmsg::flag::N;
        }
        if x.key.has_suffix() {
            header |= zmsg::flag::K;
        }
        self.write(&mut *writer, header)?;

        // Body
        self.write(&mut *writer, &x.key)?;
        self.write(&mut *writer, x.pull_id)?;
        if let Some(n) = x.max_samples {
            self.write(&mut *writer, n)?;
        }

        Ok(())
    }
}

impl<R> RCodec<Pull, &mut R> for Zenoh080
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<Pull, Self::Error> {
        let header: u8 = self.read(&mut *reader)?;
        let codec = Zenoh080Header::new(header);

        codec.read(reader)
    }
}

impl<R> RCodec<Pull, &mut R> for Zenoh080Header
where
    R: Reader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<Pull, Self::Error> {
        if imsg::mid(self.header) != zmsg::id::PULL {
            return Err(DidntRead);
        }

        let ccond = Zenoh080Condition {
            condition: imsg::has_flag(self.header, zmsg::flag::K),
            codec: self.codec,
        };
        let key: WireExpr<'static> = ccond.read(&mut *reader)?;
        let pull_id: u64 = self.codec.read(&mut *reader)?;
        let max_samples = if imsg::has_flag(self.header, zmsg::flag::N) {
            let n: u64 = self.codec.read(&mut *reader)?;
            Some(n)
        } else {
            None
        };
        let is_final = imsg::has_flag(self.header, zmsg::flag::F);

        Ok(Pull {
            key,
            pull_id,
            max_samples,
            is_final,
        })
    }
}
