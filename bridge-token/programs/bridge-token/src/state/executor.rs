use anchor_lang::prelude::*;

#[derive(Clone, Debug, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub struct MsgBody {
    pub source_token: [u8; 32],
    pub all_amount: u128,
    pub from_who: [u8; 32],
    pub to_who: [u8; 32]
}
impl MsgBody {
    pub fn to_evm_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.source_token);
        buffer.extend(&self.all_amount.to_be_bytes());
        buffer.extend_from_slice(&self.from_who);
        buffer.extend_from_slice(&self.to_who);
        buffer
    }

    pub fn from_evm_buffer(buffer: Vec<u8>) -> MsgBody {
        assert!(buffer.len() >= 112, "Buffer too short");

        let mut offset = 0;
        let source_token: [u8; 32] = buffer[offset..offset + 32]
            .try_into()
            .expect("Invalid length for source_token");
        offset += 32;

        let all_amount = u128::from_be_bytes(
            buffer[offset..offset + 16]
                .try_into()
                .expect("Invalid length for all_amount"),
        );
        offset += 16;

        let from_who: [u8; 32] = buffer[offset..offset + 32]
            .try_into()
            .expect("Invalid length for from_who");
        offset += 32;

        let to_who: [u8; 32] = buffer[offset..offset + 32]
            .try_into()
            .expect("Invalid length for to_who");
        offset += 32;
        
        MsgBody {
            source_token,
            all_amount,
            from_who,
            to_who
        }
    }
}
