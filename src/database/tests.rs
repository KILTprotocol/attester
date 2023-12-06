use sqlx::PgPool;
use uuid::Uuid;

use crate::database::dto::{Credential, Pagination, TxState};
use crate::database::querys::{
    approve_attestation_request, attestation_requests_kpis, can_approve_attestation_tx,
    can_revoke_attestation, construct_query, delete_attestation_request,
    get_attestation_request_by_id, get_attestation_requests, get_attestations_count,
    insert_attestation_request, mark_attestation_request_in_flight,
    record_attestation_request_failed, revoke_attestation_request, update_attestation_request,
};

fn get_default_attestation_request() -> Credential {
    // Create a default Credential object for testing.
    let credential_json = serde_json::json!({
        // Define claim details.
        "claim": {
            "cTypeHash": "0x3291bb126e33b4862d421bfaa1d2f272e6cdfc4f96658988fbcffea8914bd9ac",
            "contents": {
                "Email": "hello@kilt.io"
            },
            "owner": "did:kilt:4qBmSXvzSYCkTnCyqtE62KhNLrvUKvtxmkwJNQrRdMztpT1r"
        },
        // Define claimHashes and claimNonceMap.
        "claimHashes": [
            "0x2192b61d3f3109920e8991952a3fad9b7158e4fcac96dcfb873d5e975ba057e4",
            "0x2ef47f014e20bb908595f71ff022a53d7d84b5370dfed18479d4eee0575483c9"
        ],
        "claimNonceMap": {
            "0x0e0d56f241309d5a06ddf94e01d97d946f9b004d4f847302f050e5accf429c83": "5f25a0d1-b68f-4e06-a003-26c391935540",
            "0x758777288cc6705af9fb1b65f00647da18f696458ccbc59c4de0d50873e2b19d": "c57e9c72-fa8a-4e4f-b60f-a20234317bda"
        },
        // Define rootHash and claimerSignature.
        "rootHash": "0xf69ce26ca50b5d5f38cd32a99d031cd52fff42f17b9afb32895ffba260fb616a",
        "claimerSignature": {
            "keyId": "did:kilt:4siDmerNEBREZJsFoLM95x6cxEho73bCWKEDAXrKdou4a3mH#0x78579576fa15684e5d868c9e123d62d471f1a95d8f9fc8032179d3735069784d",
            "signature": "0x6243baecdfa9c752161f501597bafbb0242db1174bb8362c18d6e51bdbbdf041997fb736a07dcf56cb023687c4cc044ffba39e0dfcf01b7caa00f0f8b4fbbd81"
        },
        "legitimations": []
    });

    // Parse the JSON into a Credential object.
    serde_json::from_value::<Credential>(credential_json.clone()).unwrap()
}

#[sqlx::test]
async fn test_insert_attestation_request_valid(db_executor: PgPool) {
    // Arrange: Create a default attestation request.
    let default_credential = get_default_attestation_request();

    // Act: Insert the attestation request into the database.
    let query_result = insert_attestation_request(&default_credential, &db_executor).await;

    // Assert: Check the results of the insertion.
    assert!(query_result.is_ok());

    // Extract the inserted attestation from the result.
    let attestation = query_result.unwrap();

    // Verify that the inserted attestation has the expected properties.
    assert!(attestation.claimer == default_credential.claim.owner);
    assert!(attestation.ctype_hash == default_credential.claim.ctype_hash);
    assert!(!attestation.approved);
    assert!(!attestation.revoked);
    assert!(attestation.deleted_at.is_none());
    assert!(attestation.updated_at.is_none());
}

#[sqlx::test]
async fn test_delete_attestation_request_valid(db_executor: PgPool) {
    // Arrange: Create a default attestation request and insert it into the database.
    let default_credential = get_default_attestation_request();
    let attestation = insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Attestation creation should not fail");

    // Act: Delete the attestation request from the database.
    let query = delete_attestation_request(&attestation.id, &db_executor).await;

    // Assert: Check that the deletion was successful.
    assert!(query.is_ok());

    // Verify that the deleted attestation cannot be fetched.
    let deleted_attestation = get_attestation_request_by_id(&attestation.id, &db_executor).await;

    assert!(deleted_attestation.is_err());
}

#[sqlx::test]
async fn test_get_attestation_request_by_id_valid_id(db_executor: PgPool) {
    // Arrange: Create a default attestation request and insert it into the database.
    let default_credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Inserting attestation request should not fail");

    // Act: Get the attestation request by its valid ID.
    let result = get_attestation_request_by_id(&inserted_request.id, &db_executor).await;

    // Assert: Check that the result is successful and the retrieved request matches the inserted one.
    assert!(result.is_ok());
    let retrieved_request = result.unwrap();
    assert_eq!(retrieved_request.id, inserted_request.id);
}

#[sqlx::test]
async fn test_get_attestation_request_by_id_invalid_id(db_executor: PgPool) {
    // Arrange: Insert a default attestation request into the database.
    let default_credential = get_default_attestation_request();
    insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Inserting attestation request should not fail");

    // Act: Attempt to get an attestation request with an invalid ID.
    let invalid_id = Uuid::new_v4();
    let result = get_attestation_request_by_id(&invalid_id, &db_executor).await;

    // Assert: Check that the result is an error, as the ID doesn't exist in the database.
    assert!(result.is_err());
}

#[sqlx::test]
async fn test_get_attestations_count_with_data(db_executor: PgPool) {
    // Arrange: Insert two default attestation requests into the database.
    let default_credential = get_default_attestation_request();
    let first_attestation = insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Attestation creation should not fail");
    insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Attestation creation should not fail");

    // Act: Get the count of attestation requests in the database.
    let count = get_attestations_count(&db_executor).await;

    // Assert: Check that the count matches the number of inserted attestation requests.
    assert_eq!(count, 2);

    // Act: Delete the first attestation, and get the count again.
    delete_attestation_request(&first_attestation.id, &db_executor)
        .await
        .expect("Attestation Delete should not fail");

    // Assert: Check that the count is one less after the delete.
    let count_after_delete = get_attestations_count(&db_executor).await;
    assert_eq!(count_after_delete, 1);
}

#[sqlx::test]
async fn test_get_attestations_count_with_no_data(db_executor: PgPool) {
    // Act: Get the count of attestation requests when there's no data.
    let count = get_attestations_count(&db_executor).await;

    // Assert: Check that the count is zero.
    assert_eq!(count, 0);
}

#[sqlx::test]
async fn test_get_attestation_requests_with_data(db_executor: PgPool) {
    // Arrange: Insert some attestation requests into the database.
    let default_credential = get_default_attestation_request();
    let first_attestation = insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Attestation creation should not fail.");
    let second_attestation = insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Attestation creation should not fail.");

    // Arrange: Define pagination settings.
    let pagination = Pagination {
        offset: Some([0, 10]),
        sort: Some(["created_at".to_string(), "asc".to_string()]),
        filter: None,
    };

    // Act: Get attestation requests with the defined pagination settings.
    let query = get_attestation_requests(pagination.clone(), &db_executor).await;

    // Assert: Check that the query is successful and the results match the expectations.
    assert!(query.is_ok());
    let attestations = query.unwrap();
    assert_eq!(attestations.len(), 2);

    // Check sorting.
    assert_eq!(attestations[0].id, first_attestation.id);
    assert_eq!(attestations[1].id, second_attestation.id);

    // Check pagination
    let single_pagination = Pagination {
        offset: Some([0, 1]),
        sort: Some(["created_at".to_string(), "asc".to_string()]),
        filter: None,
    };

    let query_pagination = get_attestation_requests(single_pagination, &db_executor).await;

    assert!(query_pagination.is_ok());
    let attestations_pagination = query_pagination.unwrap();
    assert_eq!(attestations_pagination.len(), 1);

    // Delete one attestation
    delete_attestation_request(&first_attestation.id, &db_executor)
        .await
        .expect("Attestation Delete should not fail");

    // Get attestation requests after the delete
    let query_after_delete = get_attestation_requests(pagination, &db_executor).await;

    assert!(query_after_delete.is_ok());

    let attestations_after_delete = query_after_delete.unwrap();
    assert_eq!(attestations_after_delete.len(), 1);
    assert_eq!(attestations_after_delete[0].id, second_attestation.id);
}

#[sqlx::test]
async fn test_get_attestation_requests_with_no_data(db_executor: PgPool) {
    // Arrange: Define pagination settings with no offset and ascending sorting.
    let pagination = Pagination {
        offset: None,
        sort: Some(["created_at".to_string(), "asc".to_string()]),
        filter: None,
    };

    // Act: Get attestation requests with the defined pagination settings.
    let result = get_attestation_requests(pagination, &db_executor).await;

    // Assert: Check that the query is successful and there are no attestation requests.
    assert!(result.is_ok());
    let attestations = result.unwrap();
    assert!(attestations.is_empty());
}

#[test]
fn test_build_pagination_query_with_sort() {
    // Arrange: Create a pagination object with sorting by 'created_at' in descending order.
    let pagination = Pagination {
        offset: None,
        sort: Some(["created_at".to_string(), "desc".to_string()]),
        filter: None,
    };

    // Act: Build a query using the pagination settings.
    let query = construct_query(&pagination);

    // Assert: Check that the generated query matches the expected query.
    let expected_query =
        "SELECT * FROM attestation_requests WHERE deleted_at IS NULL ORDER BY $1 DESC";
    assert_eq!(query.0, expected_query);
    assert_eq!(query.1, vec!["created_at"]);
}

#[test]
fn test_build_pagination_query_with_offset() {
    // Arrange: Create a pagination object with an offset of 10 and a limit of 20.
    let pagination = Pagination {
        offset: Some([10, 20]),
        sort: None,
        filter: None,
    };

    // Act: Build a query using the pagination settings.
    let query = construct_query(&pagination);

    // Assert: Check that the generated query matches the expected query.
    let expected_query =
        "SELECT * FROM attestation_requests WHERE deleted_at IS NULL OFFSET 10 LIMIT 20";
    assert_eq!(query.0, expected_query);
    assert!(query.1.is_empty());
}
#[test]
fn test_build_pagination_query_with_sort_and_offset() {
    // Arrange: Create a pagination object with sorting by 'id' in ascending order and an offset of 5 and a limit of 15.
    let pagination = Pagination {
        offset: Some([5, 15]),
        sort: Some(["id".to_string(), "asc".to_string()]),
        filter: None,
    };

    // Act: Build a query using the pagination settings.
    let query = construct_query(&pagination);

    // Assert: Check that the generated query matches the expected query.
    let expected_query =
        "SELECT * FROM attestation_requests WHERE deleted_at IS NULL ORDER BY $1 ASC OFFSET 5 LIMIT 15";
    assert_eq!(query.0, expected_query);
    assert_eq!(query.1, vec!["id"]);
}

#[test]
fn test_build_pagination_query_no_pagination() {
    // Arrange: Create a pagination object with no sorting, no offset, and no filter.
    let pagination = Pagination {
        offset: None,
        sort: None,
        filter: None,
    };

    // Act: Build a query using the pagination settings.
    let query = construct_query(&pagination);

    // Assert: Check that the generated query matches the expected query.
    let expected_query = "SELECT * FROM attestation_requests WHERE deleted_at IS NULL";
    assert_eq!(query.0, expected_query);
    assert_eq!(query.1, Vec::<String>::new());
}

#[sqlx::test]
async fn test_update_attestation_request_valid_update(db_executor: PgPool) {
    // Arrange: Insert an attestation request into the database and create an updated credential.
    let mut default_credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Inserting attestation request should not fail");

    default_credential.root_hash = "UPDATED ROOT HASH".to_string();

    // Act: Update the attestation request with the new credential.
    let result =
        update_attestation_request(&inserted_request.id, &default_credential, &db_executor).await;

    // Assert: Check that the update is successful, and the updated credential matches the expected one.
    assert!(result.is_ok());
    let updated_request = result.unwrap();

    let updated_credential: Credential = serde_json::from_value(updated_request.credential)
        .expect("Serde JSON from value should not fail.");

    assert_eq!(updated_credential, default_credential);
}

#[sqlx::test]
async fn test_update_attestation_request_invalid_update(db_executor: PgPool) {
    // Arrange: Insert an attestation request with the default credential into the database and create an invalid ID.
    let default_credential = get_default_attestation_request();

    insert_attestation_request(&default_credential, &db_executor)
        .await
        .expect("Inserting attestation request should not fail");

    let invalid_id = Uuid::new_v4();

    // Act: Attempt to update an attestation request with an invalid ID.
    let result = update_attestation_request(&invalid_id, &default_credential, &db_executor).await;

    // Assert: Check that the update operation fails as expected.
    assert!(result.is_err());
}
#[sqlx::test]
async fn test_can_approve_attestation_tx_valid(db_executor: PgPool) {
    // Arrange: Start a transaction and insert a default attestation request.
    let mut tx = db_executor.begin().await.expect("Transaction start failed");
    let credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Act: Try to approve the attestation request.
    let result = can_approve_attestation_tx(&inserted_request.id, &mut tx).await;

    // Assert: Check that the result is successful (approval is allowed).
    assert!(result.is_ok());
}

#[sqlx::test]
async fn test_can_approve_attestation_tx_already_approved(db_executor: PgPool) {
    // Arrange: Start a transaction and insert a default attestation request.
    let mut tx = db_executor.begin().await.expect("Transaction start failed");
    let credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Mark the attestation request as already approved.
    sqlx::query!(
        "UPDATE attestation_requests SET approved = true WHERE id = $1",
        inserted_request.id
    )
    .execute(&db_executor)
    .await
    .expect("Update of attestation should not fail.");

    // Act: Try to approve the already approved attestation request.
    let result = can_approve_attestation_tx(&inserted_request.id, &mut tx).await;

    // Assert: Check that the result is an error (approval is not allowed).
    assert!(result.is_err());
}

#[sqlx::test]
async fn test_record_attestation_request_failed(db_executor: PgPool) {
    // Arrange: Start a transaction and insert a default attestation request.
    let mut tx = db_executor.begin().await.expect("Transaction start failed");
    let credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Act: Record that the attestation request has failed and commit the transaction.
    let result = record_attestation_request_failed(&inserted_request.id, &mut tx).await;

    // Assert: Check that the result is successful and the attestation request is marked as failed.
    assert!(result.is_ok());

    // Commit the transaction.
    tx.commit().await.expect("Transaction commit failed");

    // Retrieve the updated attestation request.
    let attestation = get_attestation_request_by_id(&inserted_request.id, &db_executor)
        .await
        .unwrap();

    // Check the state of the attestation request.
    assert_eq!(attestation.tx_state.unwrap(), TxState::Failed);
}

#[sqlx::test]
async fn test_can_revoke_attestation_valid(db_executor: PgPool) {
    // Arrange: Start a transaction and insert a default attestation request.
    let mut tx = db_executor.begin().await.expect("Transaction start failed");
    let credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Mark the request as approved, so it's eligible for revocation.
    sqlx::query!(
        "UPDATE attestation_requests SET approved = true WHERE id = $1",
        inserted_request.id
    )
    .execute(&db_executor)
    .await
    .expect("Update of attestation should not fail.");

    // Act: Try to revoke the approved attestation request.
    let result = can_revoke_attestation(&inserted_request.id, &mut tx).await;

    // Assert: Check that the result is successful (revocation is allowed).
    assert!(result.is_ok());
}
#[sqlx::test]
async fn test_can_revoke_attestation_already_revoked(db_executor: PgPool) {
    // Arrange: Start a transaction and insert a default attestation request.
    let mut tx = db_executor.begin().await.expect("Transaction start failed");
    let credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Mark the request as both approved and revoked.
    sqlx::query!(
        "UPDATE attestation_requests SET approved = true, revoked = true WHERE id = $1",
        inserted_request.id
    )
    .execute(&db_executor)
    .await
    .expect("Update of attestation should not fail.");

    // Act: Try to revoke an already revoked attestation request.
    let result = can_revoke_attestation(&inserted_request.id, &mut tx).await;

    // Assert: Check that the result is an error (revocation is not allowed for revoked requests).
    assert!(result.is_err());
}

#[sqlx::test]
async fn test_can_revoke_attestation_not_approved(db_executor: PgPool) {
    // Arrange: Start a transaction and insert a default attestation request.
    let mut tx = db_executor.begin().await.expect("Transaction start failed");
    let credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // The request is not marked as approved, so it should not be eligible for revocation.
    let result = can_revoke_attestation(&inserted_request.id, &mut tx).await;

    // Assert: Check that the result is an error (revocation is not allowed for unapproved requests).
    assert!(result.is_err());
}

#[sqlx::test]
async fn test_mark_attestation_request_in_flight_valid(db_executor: PgPool) {
    // Arrange: Insert a default attestation request.
    let credential = get_default_attestation_request();
    let inserted_request = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Act: Mark the attestation request as in flight.
    let result = mark_attestation_request_in_flight(&inserted_request.id, &db_executor).await;

    // Assert: Check that the result is successful.
    assert!(result.is_ok());
}

#[sqlx::test]
async fn test_attestation_requests_kpis_empty_db(db_executor: PgPool) {
    // Act: Get KPIs for an empty database.
    let result = attestation_requests_kpis(&db_executor).await;

    // Assert: Check that the result is successful and KPIs are as expected.
    assert!(result.is_ok());
    let kpis = result.expect("KPIs retrieval failed");
    assert_eq!(kpis.attestations_created_over_time.len(), 0);
    assert_eq!(kpis.attestations_not_approved, 0);
    assert_eq!(kpis.attestations_revoked, 0);
    assert_eq!(kpis.total_claimers, 0);
}

#[sqlx::test]
async fn test_attestation_requests_kpis_with_data(db_executor: PgPool) {
    // Arrange: Insert a default attestation request.
    let credential = get_default_attestation_request();
    insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Act: Get KPIs for the database with data.
    let result = attestation_requests_kpis(&db_executor).await;

    // Assert: Check that the result is successful and KPIs are as expected.
    assert!(result.is_ok());
    let kpis = result.expect("KPIs retrieval failed");
    assert_eq!(kpis.attestations_created_over_time.len(), 1);
    assert_eq!(kpis.attestations_not_approved, 1);
    assert_eq!(kpis.attestations_revoked, 0);
    assert_eq!(kpis.total_claimers, 1);
}

#[sqlx::test]
async fn test_revoke_attestation_request_valid(db_executor: PgPool) {
    // Arrange: Insert a default attestation request.
    let credential = get_default_attestation_request();
    let inserted_attestation = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Start a transaction.
    let mut tx = db_executor.begin().await.expect("Transaction start failed");

    // Act: Revoke the attestation request and commit the transaction.
    let result = revoke_attestation_request(&inserted_attestation.id, &mut tx).await;

    // Assert: Check that the result is successful, and the attestation request is revoked.
    assert!(result.is_ok());

    // Commit the transaction.
    tx.commit().await.expect("Transaction commit failed");

    // Retrieve the updated attestation request.
    let updated_attestation = get_attestation_request_by_id(&inserted_attestation.id, &db_executor)
        .await
        .expect("Attestation should exist");

    // Check the state of the attestation request.
    assert!(updated_attestation.revoked);
    assert!(updated_attestation.revoked_at.is_some());
}

#[sqlx::test]
async fn test_approve_attestation_request_valid(db_executor: PgPool) {
    // Arrange: Insert a default attestation request.
    let credential = get_default_attestation_request();
    let inserted_attestation = insert_attestation_request(&credential, &db_executor)
        .await
        .expect("Insertion failed");

    // Start a transaction.
    let mut tx = db_executor.begin().await.expect("Transaction start failed");

    // Act: Approve the attestation request and commit the transaction.
    let result = approve_attestation_request(&inserted_attestation.id, &mut tx).await;

    // Assert: Check that the result is successful, and the attestation request is approved.
    assert!(result.is_ok());

    // Commit the transaction.
    tx.commit().await.expect("Transaction commit failed");

    // Retrieve the updated attestation request.
    let updated_attestation = get_attestation_request_by_id(&inserted_attestation.id, &db_executor)
        .await
        .expect("Attestation should exist");

    // Check the state of the attestation request.
    assert!(updated_attestation.approved);
    assert!(updated_attestation.approved_at.is_some());
}
