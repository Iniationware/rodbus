use std::fmt::Display;

use crate::client::message::Promise;
use crate::common::function::FunctionCode;
use crate::decode::AppDecodeLevel;
use crate::error::RequestError;
use crate::types::{coil_from_u16, coil_to_u16, Indexed, U16Vec};

use scursor::{ReadCursor, WriteCursor};

pub(crate) trait SendBufferOperation: Sized + PartialEq {
    fn serialize(&self, cursor: &mut WriteCursor) -> Result<(), RequestError>;
    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError>;
}

pub(crate) struct SendBuffer<T>
where
    T: SendBufferOperation + Display + Send + 'static,
{
    pub(crate) request: T,
    promise: Promise<T>,
}

impl<T> SendBuffer<T>
where
    T: SendBufferOperation + Display + Send + 'static,
{
    pub(crate) fn new(request: T, promise: Promise<T>) -> Self {
        Self { request, promise }
    }

    pub(crate) fn serialize(&self, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        self.request.serialize(cursor)
    }

    pub(crate) fn failure(&mut self, err: RequestError) {
        self.promise.failure(err)
    }

    pub(crate) fn handle_response(
        &mut self,
        cursor: ReadCursor,
        function: FunctionCode,
        decode: AppDecodeLevel,
    ) -> Result<(), RequestError> {
        let response = self.parse_all(cursor)?;

        if decode.data_headers() {
            tracing::info!("PDU RX - {} {}", function, response);
        } else if decode.header() {
            tracing::info!("PDU RX - {}", function);
        }

        self.promise.success(response);
        Ok(())
    }

    fn parse_all(&self, mut cursor: ReadCursor) -> Result<T, RequestError> {
        let response = T::parse(&mut cursor)?;
        cursor.expect_empty()?;
        Ok(response)
    }
}

impl SendBufferOperation for Indexed<bool> {
    fn serialize(&self, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        cursor.write_u16_be(self.index)?;
        cursor.write_u16_be(coil_to_u16(self.value))?;
        Ok(())
    }

    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError> {
        Ok(Indexed::new(
            cursor.read_u16_be()?,
            coil_from_u16(cursor.read_u16_be()?)?,
        ))
    }
}

impl SendBufferOperation for Indexed<u16> {
    fn serialize(&self, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        cursor.write_u16_be(self.index)?;
        cursor.write_u16_be(self.value)?;
        Ok(())
    }

    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError> {
        Ok(Indexed::new(cursor.read_u16_be()?, cursor.read_u16_be()?))
    }
}

impl SendBufferOperation for U16Vec {
    fn serialize(&self, cursor: &mut WriteCursor) -> Result<(), RequestError> {
        // Writing the length of the vector first
        cursor.write_u16_be(self.len() as u16)?;
        for &item in self.iter() {
            cursor.write_u16_be(item)?;
        }
        Ok(())
    }

    fn parse(cursor: &mut ReadCursor) -> Result<Self, RequestError> {
        let len = cursor.read_u16_be()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(cursor.read_u16_be()?);
        }
        Ok(U16Vec::new(vec))  // Construct a U16Vec from the parsed vector
    }
}