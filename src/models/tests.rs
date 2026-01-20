use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::entities::tests;

/// Query parameters for listing tests
#[derive(Debug, Deserialize, IntoParams)]
pub struct TestsQuery {
    /// Filter by status: "active", "completed", "abandoned"
    pub status: Option<String>,
}

/// Parameters for creating a new test
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateTestParams {
    /// Filter type: "favorites", "category", "topic"
    #[validate(length(min = 1, max = 50))]
    pub filter_type: String,
    /// Filter ID (required for "category" and "topic" filter types)
    pub filter_id: Option<Uuid>,
    /// Language code
    #[validate(length(min = 2, max = 10))]
    pub lang: String,
    /// Number of questions (1-25)
    #[validate(range(min = 1, max = 25))]
    pub questions_count: i16,
}

/// Response for a test (list view)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TestResponse {
    pub id: Uuid,
    pub filter_type: String,
    pub filter_id: Option<Uuid>,
    pub lang: String,
    pub total_questions: i16,
    pub answered_count: i16,
    pub correct_count: i16,
    pub status: String,
    pub score_percent: Option<i16>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl TestResponse {
    pub fn from_model(model: tests::Model, answered_count: i16) -> Self {
        Self {
            id: model.id,
            filter_type: model.filter_type,
            filter_id: model.filter_id,
            lang: model.lang,
            total_questions: model.total_questions,
            answered_count,
            correct_count: model.correct_count,
            status: model.status,
            score_percent: model.score_percent,
            created_at: model.created_at.into(),
            completed_at: model.completed_at.map(|dt| dt.into()),
        }
    }
}

/// Question info in test details
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TestQuestionInfo {
    pub order: i16,
    pub question_id: Uuid,
    pub is_answered: bool,
    pub is_correct: Option<bool>,
}

/// Detailed test response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TestDetailResponse {
    pub id: Uuid,
    pub filter_type: String,
    pub filter_id: Option<Uuid>,
    pub lang: String,
    pub total_questions: i16,
    pub answered_count: i16,
    pub correct_count: i16,
    pub status: String,
    pub score_percent: Option<i16>,
    pub questions: Vec<TestQuestionInfo>,
}

/// Answer option (without is_correct for active test)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AnswerOption {
    pub id: Uuid,
    pub value: String,
}

/// Answer option with correctness (for review)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AnswerOptionWithCorrectness {
    pub id: Uuid,
    pub value: String,
    pub is_correct: bool,
}

/// Question info (without explanation for active test)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QuestionInfo {
    pub id: Uuid,
    pub name: String,
    pub content: Option<String>,
    pub lang: String,
}

/// Question info with explanation (for review)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QuestionInfoWithExplanation {
    pub id: Uuid,
    pub name: String,
    pub content: Option<String>,
    pub lang: String,
    pub explanation: String,
}

/// Current question response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CurrentQuestionResponse {
    pub order: i16,
    pub question: QuestionInfo,
    pub answers: Vec<AnswerOption>,
    pub multiple_answers: bool,
}

/// Parameters for answering a question
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct AnswerParams {
    pub question_id: Uuid,
    #[validate(length(min = 1))]
    pub answer_ids: Vec<Uuid>,
}

/// Result of answering a question
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AnswerResultResponse {
    pub is_correct: bool,
    pub correct_answer_ids: Vec<Uuid>,
    pub explanation: String,
    pub test_completed: bool,
    pub answered_count: i16,
    pub correct_count: i16,
    pub score_percent: Option<i16>,
}

/// Result of completing/abandoning a test
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CompleteTestResponse {
    pub status: String,
    pub answered_count: i16,
    pub correct_count: i16,
    pub score_percent: i16,
}

/// Question in review (with full details)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReviewQuestionResponse {
    pub order: i16,
    pub question: QuestionInfoWithExplanation,
    pub answers: Vec<AnswerOptionWithCorrectness>,
    pub selected_answer_ids: Vec<Uuid>,
    pub is_correct: bool,
}

/// Review response for completed test
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TestReviewResponse {
    pub id: Uuid,
    pub filter_type: String,
    pub filter_id: Option<Uuid>,
    pub lang: String,
    pub total_questions: i16,
    pub correct_count: i16,
    pub score_percent: i16,
    pub status: String,
    pub questions: Vec<ReviewQuestionResponse>,
}
