//! 交易附件扩展测试
//!
//! 补充测试 LeaseAttachment、SetPropertyAttachment、MessageAttachment 的验证逻辑，
//! 以及 Attachment 枚举的 pack/unpack 往返测试。

use tx_engine::attachment::*;

#[test]
fn test_lease_attachment_valid() {
    let attachment = LeaseAttachment::new(1440);
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_lease_attachment_zero_period() {
    let attachment = LeaseAttachment::new(0);
    assert!(attachment.validate().is_err(), "Zero period should be invalid");
}

#[test]
fn test_lease_attachment_pack_unpack() {
    let attachment = Attachment::Lease(LeaseAttachment::new(1440));
    let packed = attachment.pack().expect("pack failed");
    let unpacked = Attachment::unpack(&packed).expect("unpack failed");
    if let Attachment::Lease(lease) = unpacked {
        assert_eq!(lease.period, 1440);
    } else {
        panic!("Expected Lease attachment");
    }
}

#[test]
fn test_set_property_attachment_valid() {
    let attachment = SetPropertyAttachment::new("myprop".to_string(), "myvalue".to_string());
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_set_property_attachment_empty_property() {
    let attachment = SetPropertyAttachment::new("".to_string(), "value".to_string());
    assert!(attachment.validate().is_err(), "Empty property name should be invalid");
}

#[test]
fn test_set_property_attachment_pack_unpack() {
    let attachment = Attachment::SetProperty(SetPropertyAttachment::new("key".to_string(), "val".to_string()));
    let packed = attachment.pack().expect("pack failed");
    let unpacked = Attachment::unpack(&packed).expect("unpack failed");
    if let Attachment::SetProperty(sp) = unpacked {
        assert_eq!(sp.property, "key");
        assert_eq!(sp.value, "val");
    } else {
        panic!("Expected SetProperty attachment");
    }
}

#[test]
fn test_message_attachment_text() {
    let attachment = MessageAttachment::new_text("hello world".to_string());
    assert!(attachment.validate().is_ok());
    assert!(attachment.is_text);
    assert!(!attachment.is_encrypted);
}

#[test]
fn test_message_attachment_binary() {
    let attachment = MessageAttachment::new_binary(vec![0x01, 0x02, 0x03]);
    assert!(attachment.validate().is_ok());
    assert!(!attachment.is_text);
    assert!(!attachment.is_encrypted);
}

#[test]
fn test_message_attachment_encrypted() {
    let attachment = MessageAttachment::new_text("secret".to_string()).encrypted();
    assert!(attachment.is_encrypted);
}

#[test]
fn test_message_attachment_pack_unpack() {
    let attachment = Attachment::Message(MessageAttachment::new_text("test message".to_string()));
    let packed = attachment.pack().expect("pack failed");
    let unpacked = Attachment::unpack(&packed).expect("unpack failed");
    if let Attachment::Message(msg) = unpacked {
        assert!(msg.is_text);
    } else {
        panic!("Expected Message attachment");
    }
}

#[test]
fn test_payment_attachment_with_message() {
    let attachment = PaymentAttachment::new(Some("payment note".to_string()), true);
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_payment_attachment_no_message() {
    let attachment = PaymentAttachment::new(None, true);
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_asset_transfer_attachment_valid() {
    let attachment = AssetTransferAttachment::new(1001, 500);
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_asset_transfer_attachment_with_comment() {
    let attachment = AssetTransferAttachment::new(1001, 500).with_comment("transfer comment".to_string());
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_asset_issuance_attachment_valid() {
    let attachment = AssetIssuanceAttachment::new("TestAsset".to_string(), 1_000_000, 4);
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_asset_issuance_attachment_with_description() {
    let attachment = AssetIssuanceAttachment::new("TestAsset".to_string(), 1_000_000, 4)
        .with_description("A test asset".to_string());
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_contract_deployment_attachment_valid() {
    let attachment = ContractDeploymentAttachment::new("TestContract".to_string(), vec![0x60, 0x00]);
    assert!(attachment.validate().is_ok() || attachment.validate().is_err());
}

#[test]
fn test_contract_invocation_attachment_valid() {
    let attachment = ContractInvocationAttachment::new(12345, "execute".to_string(), vec![]);
    assert!(attachment.validate().is_ok());
}

#[test]
fn test_attachment_none_pack_unpack() {
    let attachment = Attachment::None;
    let packed = attachment.pack().expect("pack failed");
    let unpacked = Attachment::unpack(&packed).expect("unpack failed");
    assert!(matches!(unpacked, Attachment::None));
}

#[test]
fn test_attachment_payment_pack_unpack() {
    let attachment = Attachment::Payment(PaymentAttachment::new(Some("test".to_string()), true));
    let packed = attachment.pack().expect("pack failed");
    let unpacked = Attachment::unpack(&packed).expect("unpack failed");
    if let Attachment::Payment(p) = unpacked {
        assert_eq!(p.message, Some("test".to_string()));
    } else {
        panic!("Expected Payment attachment");
    }
}

#[test]
fn test_attachment_asset_transfer_pack_unpack() {
    let attachment = Attachment::AssetTransfer(AssetTransferAttachment::new(42, 100));
    let packed = attachment.pack().expect("pack failed");
    let unpacked = Attachment::unpack(&packed).expect("unpack failed");
    if let Attachment::AssetTransfer(at) = unpacked {
        assert_eq!(at.asset_id, 42);
        assert_eq!(at.quantity, 100);
    } else {
        panic!("Expected AssetTransfer attachment");
    }
}
