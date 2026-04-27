//! Purchase Feedback Module
//!
//! 对应 Java: PurchaseFeedback.java, PurchasePublicFeedback.java
//!
//! 处理数字商品购买的反馈

/// 加密数据
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl EncryptedData {
    pub fn new(data: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self { data, nonce }
    }
}

/// 购买反馈
#[derive(Debug, Clone)]
pub struct PurchaseFeedback {
    pub purchase_id: u64,
    pub feedback_data: Vec<u8>,
    pub feedback_nonce: Vec<u8>,
    pub height: i32,
    pub latest: bool,
}

impl PurchaseFeedback {
    pub fn new(purchase_id: u64, encrypted_data: EncryptedData, height: i32) -> Self {
        Self {
            purchase_id,
            feedback_data: encrypted_data.data,
            feedback_nonce: encrypted_data.nonce,
            height,
            latest: true,
        }
    }

    pub fn get_purchase_id(&self) -> u64 {
        self.purchase_id
    }

    pub fn get_encrypted_data(&self) -> EncryptedData {
        EncryptedData::new(self.feedback_data.clone(), self.feedback_nonce.clone())
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }
}

/// 公开购买反馈
#[derive(Debug, Clone)]
pub struct PurchasePublicFeedback {
    pub purchase_id: u64,
    pub public_feedback: Vec<u8>,
    pub height: i32,
    pub latest: bool,
}

impl PurchasePublicFeedback {
    pub fn new(purchase_id: u64, public_feedback: Vec<u8>, height: i32) -> Self {
        Self {
            purchase_id,
            public_feedback,
            height,
            latest: true,
        }
    }

    pub fn get_purchase_id(&self) -> u64 {
        self.purchase_id
    }

    pub fn get_public_feedback(&self) -> &[u8] {
        &self.public_feedback
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }
}

/// 反馈存储
pub struct FeedbackStore {
    feedbacks: Vec<PurchaseFeedback>,
    public_feedbacks: Vec<PurchasePublicFeedback>,
}

impl FeedbackStore {
    pub fn new() -> Self {
        Self {
            feedbacks: Vec::new(),
            public_feedbacks: Vec::new(),
        }
    }

    pub fn add_feedback(&mut self, feedback: PurchaseFeedback) {
        self.feedbacks.push(feedback);
    }

    pub fn add_public_feedback(&mut self, feedback: PurchasePublicFeedback) {
        self.public_feedbacks.push(feedback);
    }

    pub fn get_feedback(&self, purchase_id: u64) -> Option<&PurchaseFeedback> {
        self.feedbacks
            .iter()
            .find(|f| f.purchase_id == purchase_id && f.latest)
    }

    pub fn get_public_feedback(&self, purchase_id: u64) -> Option<&PurchasePublicFeedback> {
        self.public_feedbacks
            .iter()
            .find(|f| f.purchase_id == purchase_id && f.latest)
    }
}

impl Default for FeedbackStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_feedback() {
        let encrypted = EncryptedData::new(vec![1, 2, 3], vec![4, 5, 6]);
        let feedback = PurchaseFeedback::new(12345, encrypted, 100);

        assert_eq!(feedback.get_purchase_id(), 12345);
        assert_eq!(feedback.get_height(), 100);
    }

    #[test]
    fn test_public_feedback() {
        let feedback = PurchasePublicFeedback::new(12345, vec![1, 2, 3, 4], 100);

        assert_eq!(feedback.get_purchase_id(), 12345);
        assert_eq!(feedback.get_public_feedback(), &[1, 2, 3, 4]);
    }
}
