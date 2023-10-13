use sqlx::PgPool;
use uuid::Uuid;

use crate::database::dto::{AttestationRequest, Credential, Pagination};
use crate::database::querys::{
    build_pagination_query, delete_attestation_request, get_attestation_request_by_id,
    get_attestation_requests, get_attestations_count, insert_attestation_request,
    update_attestation_request,
};

fn get_default_attestation_request() -> (AttestationRequest, Credential) {
    let credential_json = serde_json::json!({
        "claim": {
            "cTypeHash": "0x3291bb126e33b4862d421bfaa1d2f272e6cdfc4f96658988fbcffea8914bd9ac",
            "contents": {
                "Email": "hello@kilt.io"
            },
            "owner": "did:kilt:4qBmSXvzSYCkTnCyqtE62KhNLrvUKvtxmkwJNQrRdMztpT1r"
        },
        "claimHashes": [
            "0x2192b61d3f3109920e8991952a3fad9b7158e4fcac96dcfb873d5e975ba057e4",
            "0x2ef47f014e20bb908595f71ff022a53d7d84b5370dfed18479d4eee0575483c9"
        ],
        "claimNonceMap": {
            "0x0e0d56f241309d5a06ddf94e01d97d946f9b004d4f847302f050e5accf429c83": "5f25a0d1-b68f-4e06-a003-26c391935540",
            "0x758777288cc6705af9fb1b65f00647da18f696458ccbc59c4de0d50873e2b19d": "c57e9c72-fa8a-4e4f-b60f-a20234317bda"
        },
        "rootHash": "0xf69ce26ca50b5d5f38cd32a99d031cd52fff42f17b9afb32895ffba260fb616a",
        "claimerSignature": {
            "keyId": "did:kilt:4siDmerNEBREZJsFoLM95x6cxEho73bCWKEDAXrKdou4a3mH#0x78579576fa15684e5d868c9e123d62d471f1a95d8f9fc8032179d3735069784d",
            "signature": "0x6243baecdfa9c752161f501597bafbb0242db1174bb8362c18d6e51bdbbdf041997fb736a07dcf56cb023687c4cc044ffba39e0dfcf01b7caa00f0f8b4fbbd81"
        },
        "legitimations": []
    });

    let credential = serde_json::from_value::<Credential>(credential_json.clone()).unwrap();
    let claimer = "did:kilt:4qBmSXvzSYCkTnCyqtE62KhNLrvUKvtxmkwJNQrRdMztpT1r".to_string();
    let ctype_hash =
        "0x3291bb126e33b4862d421bfaa1d2f272e6cdfc4f96658988fbcffea8914bd9ac".to_string();

    (
        AttestationRequest {
            ctype_hash: ctype_hash.clone(),
            claimer: claimer.clone(),
            credential: credential_json,
        },
        credential,
    )
}

#[sqlx::test]
async fn check_attestation_requests_can_be_created(db_executor: PgPool) {
    let (default_attesation, default_credential) = get_default_attestation_request();

    let query_result =
        insert_attestation_request(&default_attesation, &default_credential, &db_executor).await;

    assert!(query_result.is_ok());

    let attestation = query_result.unwrap();

    assert!(attestation.claimer == default_attesation.claimer);
    assert!(attestation.ctype_hash == default_attesation.ctype_hash);

    assert!(!attestation.approved);
    assert!(!attestation.revoked);
    assert!(attestation.deleted_at.is_none());
    assert!(attestation.updated_at.is_none());
}

#[sqlx::test]
async fn check_attestation_requests_can_be_deleted(db_executor: PgPool) {
    let (default_attesation, default_credential) = get_default_attestation_request();

    let attesatation =
        insert_attestation_request(&default_attesation, &default_credential, &db_executor)
            .await
            .expect("Attesatation creation should not fail");

    let query = delete_attestation_request(&attesatation.id, &db_executor).await;

    assert!(query.is_ok());

    // we should not be able to fetch it the deleted attesattion
    let deleted_attesation = get_attestation_request_by_id(&attesatation.id, &db_executor).await;

    assert!(deleted_attesation.is_err());
}

#[sqlx::test]
async fn test_get_attestation_request_by_id_valid_id(db_executor: PgPool) {
    let (default_attestation, default_credential) = get_default_attestation_request();
    let inserted_request =
        insert_attestation_request(&default_attestation, &default_credential, &db_executor)
            .await
            .expect("Inserting attestation request should not fail");

    let result = get_attestation_request_by_id(&inserted_request.id, &db_executor).await;

    assert!(result.is_ok());
    let retrieved_request = result.unwrap();
    assert_eq!(retrieved_request.id, inserted_request.id);
}

#[sqlx::test]
async fn test_get_attestation_request_by_id_invalid_id(db_executor: PgPool) {
    let (default_attestation, default_credential) = get_default_attestation_request();
    insert_attestation_request(&default_attestation, &default_credential, &db_executor)
        .await
        .expect("Inserting attestation request should not fail");

    let invalid_id = Uuid::new_v4();
    let result = get_attestation_request_by_id(&invalid_id, &db_executor).await;

    assert!(result.is_err());
}

#[sqlx::test]
async fn test_get_attestations_count_with_data(db_executor: PgPool) {
    let (default_attestation, default_credential) = get_default_attestation_request();

    let first_attestation =
        insert_attestation_request(&default_attestation, &default_credential, &db_executor)
            .await
            .expect("Attesatation creation should not fail");

    insert_attestation_request(&default_attestation, &default_credential, &db_executor)
        .await
        .expect("Attesatation creation should not fail");

    let count = get_attestations_count(&db_executor).await;

    assert_eq!(count, 2);

    delete_attestation_request(&first_attestation.id, &db_executor)
        .await
        .expect("Attestation Delete should not fail");

    // after the delete the count should be one less

    let count_after_delete = get_attestations_count(&db_executor).await;

    assert_eq!(count_after_delete, 1);
}

#[sqlx::test]
async fn test_get_attestations_count_with_no_data(db_executor: PgPool) {
    let count = get_attestations_count(&db_executor).await;
    assert_eq!(count, 0);
}

#[sqlx::test]
async fn test_get_attestation_requests_with_data(db_executor: PgPool) {
    // Arrange: Insert some attestation requests into the database.
    let (default_attestation, default_credential) = get_default_attestation_request();
    let first_attestation =
        insert_attestation_request(&default_attestation, &default_credential, &db_executor)
            .await
            .expect("Attestation creation should not fail.");
    let second_attestattion =
        insert_attestation_request(&default_attestation, &default_credential, &db_executor)
            .await
            .expect("Attestation creation should not fail.");

    let pagination = Pagination {
        offset: Some([10, 0]),
        sort: Some(["created_at".to_string(), "asc".to_string()]),
    };

    let query = get_attestation_requests(&pagination, &db_executor).await;

    assert!(query.is_ok());
    let attesatations = query.unwrap();
    assert_eq!(attesatations.len(), 2);

    //check sorting.
    assert_eq!(attesatations[0].id, first_attestation.id);
    assert_eq!(attesatations[1].id, second_attestattion.id);

    // check pagination
    let signle_pagination = Pagination {
        offset: Some([1, 0]),
        sort: Some(["created_at".to_string(), "asc".to_string()]),
    };

    let query_pagination = get_attestation_requests(&signle_pagination, &db_executor).await;

    assert!(query_pagination.is_ok());
    let attesatations_pagination = query_pagination.unwrap();
    assert_eq!(attesatations_pagination.len(), 1);

    // Delete one attesatation
    delete_attestation_request(&first_attestation.id, &db_executor)
        .await
        .expect("Attestation Delete should not fail");

    let query_after_delete = get_attestation_requests(&pagination, &db_executor).await;

    assert!(query_after_delete.is_ok());

    let attestations_after_delete = query_after_delete.unwrap();
    assert_eq!(attestations_after_delete.len(), 1);

    assert_eq!(attestations_after_delete[0].id, second_attestattion.id)
}

#[sqlx::test]
async fn test_get_attestation_requests_with_no_data(db_executor: PgPool) {
    let pagination = Pagination {
        offset: Some([10, 0]),
        sort: Some(["created_at".to_string(), "asc".to_string()]),
    };

    let result = get_attestation_requests(&pagination, &db_executor).await;

    assert!(result.is_ok());
    let attestations = result.unwrap();
    assert!(attestations.is_empty());
}

#[test]
fn test_build_pagination_query_with_sort() {
    let pagination = Pagination {
        offset: None,
        sort: Some(["created_at".to_string(), "desc".to_string()]),
    };

    let query = build_pagination_query(&pagination);

    let expected_query =
        "SELECT * FROM attestation_requests WHERE deleted_at IS NULL ORDER BY created_at desc ";
    assert_eq!(query, expected_query);
}
#[test]
fn test_build_pagination_query_with_offset() {
    let pagination = Pagination {
        offset: Some([10, 20]),
        sort: None,
    };

    let query = build_pagination_query(&pagination);

    let expected_query =
        "SELECT * FROM attestation_requests WHERE deleted_at IS NULL LIMIT 10 OFFSET 20";
    assert_eq!(query, expected_query);
}

#[test]
fn test_build_pagination_query_with_sort_and_offset() {
    let pagination = Pagination {
        offset: Some([5, 15]),
        sort: Some(["id".to_string(), "asc".to_string()]),
    };

    let query = build_pagination_query(&pagination);

    let expected_query = "SELECT * FROM attestation_requests WHERE deleted_at IS NULL ORDER BY id asc LIMIT 5 OFFSET 15";
    assert_eq!(query, expected_query);
}

#[test]
fn test_build_pagination_query_no_pagination() {
    let pagination = Pagination {
        offset: None,
        sort: None,
    };

    let query = build_pagination_query(&pagination);

    let expected_query = "SELECT * FROM attestation_requests WHERE deleted_at IS NULL ";
    assert_eq!(query, expected_query);
}

#[sqlx::test]
async fn test_update_attestation_request_valid_update(db_executor: PgPool) {
    let (default_attestation, mut default_credential) = get_default_attestation_request();
    let inserted_request =
        insert_attestation_request(&default_attestation, &default_credential, &db_executor)
            .await
            .expect("Inserting attestation request should not fail");

    default_credential.root_hash = "UPDATED ROOT HASH".to_string();

    let result =
        update_attestation_request(&inserted_request.id, &default_credential, &db_executor).await;

    assert!(result.is_ok());
    let updated_request = result.unwrap();
    assert_eq!(
        updated_request.credential,
        serde_json::to_value(&default_credential).unwrap()
    );
}

#[sqlx::test]
async fn test_update_attestation_request_invalid_update(db_executor: PgPool) {
    let (default_attestation, default_credential) = get_default_attestation_request();

    insert_attestation_request(&default_attestation, &default_credential, &db_executor)
        .await
        .expect("Inserting attestation request should not fail");

    let invalid_id = Uuid::new_v4();

    let result = update_attestation_request(&invalid_id, &default_credential, &db_executor).await;

    assert!(result.is_err());
}
