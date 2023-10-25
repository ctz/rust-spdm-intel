// Copyright (c) 2022 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 or MIT

use spdmlib::error::*;

use crate::context::{MessagePayloadRequestStopInterface, MessagePayloadResponseStopInterface};

use super::*;

// security check
// If the interface ID in the request is not hosted by the device.

// Abort all in-flight and accepted operations that are being performed by the TDI
// Wait for outstanding responses for the aborted operations
// All DMA read and write operations by the TDI are aborted or completed
// All interrupts from the TDI have been generated
// If function hosting the TDI is capable of Address Translation Service (ATS), all ATS requests by the TDI have completed or aborted. All translations cached in the device for ATS requests generated by this TDI have been invalidated.
// If function hosting the TDI is capable of Page Request Interface Service (PRI), no more page requests will be generated by the TDI. Additionally, either page responses have been received for all page requests generated by the TDI or the TDI will discard page responses for outstanding page requests.
// Scrub internal state of the device to remove secrets associated with the TDI such that those secrets will not be accessible.
// Reclaim and scrub private resources (e.g., memory encryption keys for device attached memories, etc.) assigned to the TDI

impl<'a> TdispResponder<'a> {
    pub fn handle_stop_interface_request(
        &mut self,
        vendor_defined_req_payload_struct: &VendorDefinedReqPayloadStruct,
    ) -> SpdmResult<VendorDefinedRspPayloadStruct> {
        let mut reader =
            Reader::init(&vendor_defined_req_payload_struct.vendor_defined_req_payload);
        let tmh = TdispMessageHeader::tdisp_read(&mut self.tdisp_requester_context, &mut reader);
        let mpr = MessagePayloadRequestStopInterface::tdisp_read(
            &mut self.tdisp_requester_context,
            &mut reader,
        );
        if tmh.is_none() || mpr.is_none() {
            self.handle_tdisp_error(
                vendor_defined_req_payload_struct,
                MESSAGE_PAYLOAD_RESPONSE_TDISP_ERROR_INVALID_REQUEST,
            )
        } else {
            let mut vendor_defined_rsp_payload_struct: VendorDefinedRspPayloadStruct =
                VendorDefinedRspPayloadStruct {
                    rsp_length: 0,
                    vendor_defined_rsp_payload: [0u8;
                        spdmlib::config::MAX_SPDM_VENDOR_DEFINED_PAYLOAD_SIZE],
                };
            let mut writer =
                Writer::init(&mut vendor_defined_rsp_payload_struct.vendor_defined_rsp_payload);

            let tmhr = TdispMessageHeader {
                tdisp_version: self.tdisp_requester_context.version_sel,
                message_type: TdispRequestResponseCode::ResponseStopInterfaceResponse,
                interface_id: self.tdisp_requester_context.tdi,
            };

            let mprr = MessagePayloadResponseStopInterface::default();

            tmhr.tdisp_encode(&mut self.tdisp_requester_context, &mut writer);
            mprr.tdisp_encode(&mut self.tdisp_requester_context, &mut writer);

            match self
                .tdisp_requester_context
                .configuration
                .erase_confidential_config()
            {
                Ok(_) => match self.tdisp_requester_context.configuration.unlock_config() {
                    Ok(_) => {
                        self.tdisp_requester_context
                            .state_machine
                            .to_state_config_unlocked();
                        Ok(vendor_defined_rsp_payload_struct)
                    }
                    Err(_) => self.handle_tdisp_error(
                        vendor_defined_req_payload_struct,
                        MESSAGE_PAYLOAD_RESPONSE_TDISP_ERROR_INVALID_DEVICE_CONFIGURATION,
                    ),
                },
                Err(_) => self.handle_tdisp_error(
                    vendor_defined_req_payload_struct,
                    MESSAGE_PAYLOAD_RESPONSE_TDISP_ERROR_INVALID_DEVICE_CONFIGURATION,
                ),
            }
        }
    }
}
