#[path = "../../service/tests/support/database_contract.rs"]
mod shared;
#[tokio::test]
async fn memory_contract() {
    shared::contract(std::sync::Arc::new(database_memory::MemoryDatabase::new())).await;
}
