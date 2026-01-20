use crate::{
    AppContext,
    entities::{
        answers, categories, question_categories, questions, test_question_answers, test_questions,
        tests, topics, user_favorite_questions,
    },
    models::tests::{
        AnswerOption, AnswerOptionWithCorrectness, AnswerParams, AnswerResultResponse,
        CompleteTestResponse, CreateTestParams, CurrentQuestionResponse, QuestionInfo,
        QuestionInfoWithExplanation, ReviewQuestionResponse, TestDetailResponse, TestQuestionInfo,
        TestResponse, TestReviewResponse, TestsQuery,
    },
    utils::{extractors::AuthUser, response::ApiError},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use rand::seq::SliceRandom;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, TransactionTrait,
};
use std::collections::HashSet;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;
use validator::Validate;

/// Helper to generate filter_hash
fn generate_filter_hash(filter_type: &str, filter_id: Option<Uuid>, lang: &str) -> String {
    match filter_id {
        Some(id) => format!("{}:{}:{}", filter_type, id, lang),
        None => format!("{}:{}", filter_type, lang),
    }
}

/// List user's tests
#[utoipa::path(
    get,
    tag = "Tests",
    path = "/api/tests",
    params(TestsQuery),
    responses(
        (status = 200, body = Vec<TestResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn list(
    auth_user: AuthUser,
    State(ctx): State<AppContext>,
    Query(query): Query<TestsQuery>,
) -> axum::response::Result<Response> {
    let mut tests_query = tests::Entity::find()
        .filter(tests::Column::UserId.eq(auth_user.user.id))
        .filter(tests::Column::IsDeleted.eq(false));

    if let Some(status) = query.status {
        tests_query = tests_query.filter(tests::Column::Status.eq(status));
    }

    let tests_list = tests_query
        .order_by_desc(tests::Column::CreatedAt)
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    let mut responses = Vec::new();
    for test in tests_list {
        let answered_count = test_questions::Entity::find()
            .filter(test_questions::Column::TestId.eq(test.id))
            .filter(test_questions::Column::AnsweredAt.is_not_null())
            .all(&ctx.db)
            .await
            .map_err(ApiError::from)?
            .len() as i16;

        responses.push(TestResponse::from_model(test, answered_count));
    }

    Ok(Json(responses).into_response())
}

/// Get test details
#[utoipa::path(
    get,
    tag = "Tests",
    path = "/api/tests/{id}",
    params(("id" = Uuid, Path, description = "Test ID")),
    responses(
        (status = 200, body = TestDetailResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let test = tests::Entity::find_by_id(id)
        .filter(tests::Column::IsDeleted.eq(false))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    if test.user_id != auth_user.user.id {
        return Err(ApiError::Forbidden.into());
    }

    let test_questions_list = test_questions::Entity::find()
        .filter(test_questions::Column::TestId.eq(test.id))
        .order_by_asc(test_questions::Column::QuestionOrder)
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    let questions: Vec<TestQuestionInfo> = test_questions_list
        .iter()
        .map(|tq| TestQuestionInfo {
            order: tq.question_order,
            question_id: tq.question_id,
            is_answered: tq.answered_at.is_some(),
            is_correct: tq.is_correct,
        })
        .collect();

    let answered_count = questions.iter().filter(|q| q.is_answered).count() as i16;

    Ok(Json(TestDetailResponse {
        id: test.id,
        filter_type: test.filter_type,
        filter_id: test.filter_id,
        lang: test.lang,
        total_questions: test.total_questions,
        answered_count,
        correct_count: test.correct_count,
        status: test.status,
        score_percent: test.score_percent,
        questions,
    })
    .into_response())
}

/// Create a new test
#[utoipa::path(
    post,
    tag = "Tests",
    path = "/api/tests",
    request_body = CreateTestParams,
    responses(
        (status = 201, body = TestResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn create(
    auth_user: AuthUser,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateTestParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;

    // Validate filter_type
    if !["favorites", "category", "topic"].contains(&params.filter_type.as_str()) {
        return Err(ApiError::InvalidFieldValue.into());
    }

    // Validate filter_id requirement
    if params.filter_type != "favorites" && params.filter_id.is_none() {
        return Err(ApiError::MissingField.into());
    }

    let filter_hash = generate_filter_hash(&params.filter_type, params.filter_id, &params.lang);

    // Check for existing active test with same filter
    let existing = tests::Entity::find()
        .filter(tests::Column::UserId.eq(auth_user.user.id))
        .filter(tests::Column::FilterHash.eq(&filter_hash))
        .filter(tests::Column::Status.eq("active"))
        .filter(tests::Column::IsDeleted.eq(false))
        .filter(tests::Column::IsActive.eq(true))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    if existing.is_some() {
        return Err(ApiError::AlreadyExists.into());
    }

    // Get questions based on filter_type
    let question_ids: Vec<Uuid> = match params.filter_type.as_str() {
        "favorites" => {
            let favorites = user_favorite_questions::Entity::find()
                .filter(user_favorite_questions::Column::UserId.eq(auth_user.user.id))
                .all(&ctx.db)
                .await
                .map_err(ApiError::from)?;

            let fav_question_ids: Vec<Uuid> = favorites.iter().map(|f| f.question_id).collect();

            questions::Entity::find()
                .filter(questions::Column::Id.is_in(fav_question_ids))
                .filter(questions::Column::Lang.eq(&params.lang))
                .all(&ctx.db)
                .await
                .map_err(ApiError::from)?
                .iter()
                .map(|q| q.id)
                .collect()
        }
        "category" => {
            let category_id = params.filter_id.unwrap();

            // Check category exists
            categories::Entity::find_by_id(category_id)
                .one(&ctx.db)
                .await
                .map_err(ApiError::from)?
                .ok_or(ApiError::NotFound)?;

            let qc = question_categories::Entity::find()
                .filter(question_categories::Column::CategoryId.eq(category_id))
                .all(&ctx.db)
                .await
                .map_err(ApiError::from)?;

            let qc_question_ids: Vec<Uuid> = qc.iter().map(|qc| qc.question_id).collect();

            questions::Entity::find()
                .filter(questions::Column::Id.is_in(qc_question_ids))
                .filter(questions::Column::Lang.eq(&params.lang))
                .all(&ctx.db)
                .await
                .map_err(ApiError::from)?
                .iter()
                .map(|q| q.id)
                .collect()
        }
        "topic" => {
            let topic_id = params.filter_id.unwrap();

            // Check topic exists
            topics::Entity::find_by_id(topic_id)
                .one(&ctx.db)
                .await
                .map_err(ApiError::from)?
                .ok_or(ApiError::NotFound)?;

            questions::Entity::find()
                .filter(questions::Column::TopicId.eq(topic_id))
                .filter(questions::Column::Lang.eq(&params.lang))
                .all(&ctx.db)
                .await
                .map_err(ApiError::from)?
                .iter()
                .map(|q| q.id)
                .collect()
        }
        _ => return Err(ApiError::InvalidFieldValue.into()),
    };

    // Check if we have enough questions
    if (question_ids.len() as i16) < params.questions_count {
        return Err(ApiError::InvalidInput.into());
    }

    // Select random questions (scope rng to avoid Send issues)
    let selected_ids: Vec<Uuid> = {
        let mut rng = rand::thread_rng();
        let mut ids = question_ids.clone();
        ids.shuffle(&mut rng);
        ids.into_iter()
            .take(params.questions_count as usize)
            .collect()
    };

    // Create test
    let test = tests::ActiveModel {
        user_id: Set(auth_user.user.id),
        filter_type: Set(params.filter_type),
        filter_id: Set(params.filter_id),
        lang: Set(params.lang),
        filter_hash: Set(filter_hash),
        total_questions: Set(params.questions_count),
        correct_count: Set(0),
        status: Set("active".to_string()),
        ..Default::default()
    };

    let test = test.insert(&ctx.db).await.map_err(ApiError::from)?;

    // Create test_questions
    for (order, question_id) in selected_ids.iter().enumerate() {
        let tq = test_questions::ActiveModel {
            test_id: Set(test.id),
            question_id: Set(*question_id),
            question_order: Set((order + 1) as i16),
            is_correct: Set(None),
            answered_at: Set(None),
        };
        tq.insert(&ctx.db).await.map_err(ApiError::from)?;
    }

    Ok((
        axum::http::StatusCode::CREATED,
        Json(TestResponse::from_model(test, 0)),
    )
        .into_response())
}

/// Get current (next unanswered) question
#[utoipa::path(
    get,
    tag = "Tests",
    path = "/api/tests/{id}/current",
    params(("id" = Uuid, Path, description = "Test ID")),
    responses(
        (status = 200, body = CurrentQuestionResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn get_current_question(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let test = tests::Entity::find_by_id(id)
        .filter(tests::Column::IsDeleted.eq(false))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    if test.user_id != auth_user.user.id {
        return Err(ApiError::Forbidden.into());
    }

    if test.status != "active" {
        return Err(ApiError::InvalidState.into());
    }

    // Find first unanswered question
    let next_question = test_questions::Entity::find()
        .filter(test_questions::Column::TestId.eq(test.id))
        .filter(test_questions::Column::AnsweredAt.is_null())
        .order_by_asc(test_questions::Column::QuestionOrder)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    let tq = match next_question {
        Some(tq) => tq,
        None => return Err(ApiError::NotFound.into()),
    };

    // Get question
    let question = questions::Entity::find_by_id(tq.question_id)
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Get answers
    let answers_list = answers::Entity::find()
        .filter(answers::Column::QuestionId.eq(question.id))
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    // Count correct answers to determine if multiple selection
    let correct_count = answers_list.iter().filter(|a| a.is_correct).count();

    Ok(Json(CurrentQuestionResponse {
        order: tq.question_order,
        question: QuestionInfo {
            id: question.id,
            name: question.name,
            content: question.content,
            lang: question.lang,
        },
        answers: answers_list
            .iter()
            .map(|a| AnswerOption {
                id: a.id,
                value: a.value.clone(),
            })
            .collect(),
        multiple_answers: correct_count > 1,
    })
    .into_response())
}

/// Answer a question
#[utoipa::path(
    post,
    tag = "Tests",
    path = "/api/tests/{id}/answer",
    params(("id" = Uuid, Path, description = "Test ID")),
    request_body = AnswerParams,
    responses(
        (status = 200, body = AnswerResultResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn answer_question(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
    Json(params): Json<AnswerParams>,
) -> axum::response::Result<Response> {
    params.validate().map_err(ApiError::from)?;
    let txn = ctx.db.begin().await.map_err(ApiError::from)?;

    let test = tests::Entity::find_by_id(id)
        .filter(tests::Column::IsDeleted.eq(false))
        .one(&txn)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    if test.user_id != auth_user.user.id {
        return Err(ApiError::Forbidden.into());
    }

    if test.status != "active" {
        return Err(ApiError::InvalidState.into());
    }

    // Find the question in test
    let tq = test_questions::Entity::find()
        .filter(test_questions::Column::TestId.eq(test.id))
        .filter(test_questions::Column::QuestionId.eq(params.question_id))
        .one(&txn)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    if tq.answered_at.is_some() {
        return Err(ApiError::Conflict.into());
    }

    // Get question for explanation
    let question = questions::Entity::find_by_id(params.question_id)
        .one(&txn)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    // Get all correct answers for the question
    let correct_answers = answers::Entity::find()
        .filter(answers::Column::QuestionId.eq(params.question_id))
        .filter(answers::Column::IsCorrect.eq(true))
        .all(&txn)
        .await
        .map_err(ApiError::from)?;

    let correct_ids: HashSet<Uuid> = correct_answers.iter().map(|a| a.id).collect();
    let selected_ids: HashSet<Uuid> = params.answer_ids.iter().cloned().collect();

    // Answer is correct only if selected_ids == correct_ids (exact match)
    let is_correct = selected_ids == correct_ids;

    // Save selected answers to test_question_answers
    for answer_id in &params.answer_ids {
        let tqa = test_question_answers::ActiveModel {
            test_id: Set(test.id),
            question_id: Set(params.question_id),
            answer_id: Set(*answer_id),
        };
        tqa.insert(&txn).await.map_err(ApiError::from)?;
    }

    // Update test_questions
    let mut tq_active = tq.into_active_model();
    tq_active.is_correct = Set(Some(is_correct));
    tq_active.answered_at = Set(Some(chrono::Utc::now().into()));
    tq_active.update(&txn).await.map_err(ApiError::from)?;

    // Update test correct_count if correct
    let mut test_active = test.clone().into_active_model();
    if is_correct {
        test_active.correct_count = Set(test.correct_count + 1);
    }

    // Check if all questions are answered
    let unanswered = test_questions::Entity::find()
        .filter(test_questions::Column::TestId.eq(test.id))
        .filter(test_questions::Column::AnsweredAt.is_null())
        .all(&txn)
        .await
        .map_err(ApiError::from)?;

    let test_completed = unanswered.len() == 0; // Current question was the last one
    let new_correct_count = if is_correct {
        test.correct_count + 1
    } else {
        test.correct_count
    };
    let answered_count = test.total_questions - (unanswered.len() as i16 - 1);

    let score_percent = if test_completed {
        let score = (new_correct_count as f32 / test.total_questions as f32 * 100.0) as i16;
        test_active.status = Set("completed".to_string());
        test_active.score_percent = Set(Some(score));
        test_active.completed_at = Set(Some(chrono::Utc::now().into()));
        Some(score)
    } else {
        None
    };

    test_active.update(&txn).await.map_err(ApiError::from)?;

    txn.commit().await.map_err(ApiError::from)?;

    Ok(Json(AnswerResultResponse {
        is_correct,
        correct_answer_ids: correct_ids.into_iter().collect(),
        explanation: question.explanation,
        test_completed,
        answered_count,
        correct_count: new_correct_count,
        score_percent,
    })
    .into_response())
}

/// Force complete (abandon) a test
#[utoipa::path(
    post,
    tag = "Tests",
    path = "/api/tests/{id}/complete",
    params(("id" = Uuid, Path, description = "Test ID")),
    responses(
        (status = 200, body = CompleteTestResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn complete_test(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let test = tests::Entity::find_by_id(id)
        .filter(tests::Column::IsDeleted.eq(false))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    if test.user_id != auth_user.user.id {
        return Err(ApiError::Forbidden.into());
    }

    if test.status != "active" {
        return Err(ApiError::InvalidState.into());
    }

    // Count answered questions
    let answered_count = test_questions::Entity::find()
        .filter(test_questions::Column::TestId.eq(test.id))
        .filter(test_questions::Column::AnsweredAt.is_not_null())
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .len() as i16;

    // Calculate score based on answered questions
    let score_percent = if answered_count > 0 {
        (test.correct_count as f32 / answered_count as f32 * 100.0) as i16
    } else {
        0
    };

    let mut test_active = test.into_active_model();
    test_active.status = Set("abandoned".to_string());
    test_active.score_percent = Set(Some(score_percent));
    test_active.completed_at = Set(Some(chrono::Utc::now().into()));
    let updated_test = test_active.update(&ctx.db).await.map_err(ApiError::from)?;

    Ok(Json(CompleteTestResponse {
        status: updated_test.status,
        answered_count,
        correct_count: updated_test.correct_count,
        score_percent,
    })
    .into_response())
}

/// Delete (soft delete) a test
#[utoipa::path(
    delete,
    tag = "Tests",
    path = "/api/tests/{id}",
    params(("id" = Uuid, Path, description = "Test ID")),
    responses(
        (status = 200),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn delete(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let test = tests::Entity::find_by_id(id)
        .filter(tests::Column::IsDeleted.eq(false))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    if test.user_id != auth_user.user.id {
        return Err(ApiError::Forbidden.into());
    }

    let mut test_active = test.into_active_model();
    test_active.is_deleted = Set(true);
    test_active.update(&ctx.db).await.map_err(ApiError::from)?;

    Ok(().into_response())
}

/// Get history of completed tests
#[utoipa::path(
    get,
    tag = "Tests",
    path = "/api/tests/history",
    responses(
        (status = 200, body = Vec<TestResponse>),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn history(
    auth_user: AuthUser,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let tests_list = tests::Entity::find()
        .filter(tests::Column::UserId.eq(auth_user.user.id))
        .filter(tests::Column::IsDeleted.eq(false))
        .filter(tests::Column::Status.ne("active"))
        .order_by_desc(tests::Column::CompletedAt)
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    let responses: Vec<TestResponse> = tests_list
        .into_iter()
        .map(|test| {
            let answered = test.total_questions; // For completed tests, assume all are answered
            TestResponse::from_model(test, answered)
        })
        .collect();

    Ok(Json(responses).into_response())
}

/// Review a completed test
#[utoipa::path(
    get,
    tag = "Tests",
    path = "/api/tests/{id}/review",
    params(("id" = Uuid, Path, description = "Test ID")),
    responses(
        (status = 200, body = TestReviewResponse),
        ApiError
    ),
    security(("jwt_token" = []))
)]
async fn review(
    auth_user: AuthUser,
    Path(id): Path<Uuid>,
    State(ctx): State<AppContext>,
) -> axum::response::Result<Response> {
    let test = tests::Entity::find_by_id(id)
        .filter(tests::Column::IsDeleted.eq(false))
        .one(&ctx.db)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    if test.user_id != auth_user.user.id {
        return Err(ApiError::Forbidden.into());
    }

    if test.status == "active" {
        return Err(ApiError::InvalidState.into());
    }

    let test_questions_list = test_questions::Entity::find()
        .filter(test_questions::Column::TestId.eq(test.id))
        .order_by_asc(test_questions::Column::QuestionOrder)
        .all(&ctx.db)
        .await
        .map_err(ApiError::from)?;

    let mut review_questions = Vec::new();

    for tq in test_questions_list {
        // Get question
        let question = questions::Entity::find_by_id(tq.question_id)
            .one(&ctx.db)
            .await
            .map_err(ApiError::from)?
            .ok_or(ApiError::NotFound)?;

        // Get all answers
        let answers_list = answers::Entity::find()
            .filter(answers::Column::QuestionId.eq(question.id))
            .all(&ctx.db)
            .await
            .map_err(ApiError::from)?;

        // Get selected answers
        let selected_answers = test_question_answers::Entity::find()
            .filter(test_question_answers::Column::TestId.eq(test.id))
            .filter(test_question_answers::Column::QuestionId.eq(question.id))
            .all(&ctx.db)
            .await
            .map_err(ApiError::from)?;

        let selected_ids: Vec<Uuid> = selected_answers.iter().map(|sa| sa.answer_id).collect();

        review_questions.push(ReviewQuestionResponse {
            order: tq.question_order,
            question: QuestionInfoWithExplanation {
                id: question.id,
                name: question.name,
                content: question.content,
                lang: question.lang,
                explanation: question.explanation,
            },
            answers: answers_list
                .iter()
                .map(|a| AnswerOptionWithCorrectness {
                    id: a.id,
                    value: a.value.clone(),
                    is_correct: a.is_correct,
                })
                .collect(),
            selected_answer_ids: selected_ids,
            is_correct: tq.is_correct.unwrap_or(false),
        });
    }

    Ok(Json(TestReviewResponse {
        id: test.id,
        filter_type: test.filter_type,
        filter_id: test.filter_id,
        lang: test.lang,
        total_questions: test.total_questions,
        correct_count: test.correct_count,
        score_percent: test.score_percent.unwrap_or(0),
        status: test.status,
        questions: review_questions,
    })
    .into_response())
}

pub fn routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(get))
        .routes(routes!(create))
        .routes(routes!(get_current_question))
        .routes(routes!(answer_question))
        .routes(routes!(complete_test))
        .routes(routes!(delete))
        .routes(routes!(history))
        .routes(routes!(review))
}
