use crate::generated::Person;
use catalytic::env_property_reader::database_url;
use catalytic::query_transform::SelectMultiple;
use scylla::frame::value::SerializeValuesError;
use scylla::statement::Consistency;
use scylla::transport::errors::QueryError;
use scylla::{CachingSession, SessionBuilder};

mod generated;

#[tokio::main]
async fn main() {
    let session = create_session().await;

    crud_person(&session).await.unwrap();
    compile_time_checked_query(&session).await;
}

/// Demonstrates an easy way how to write a compile time checked query
async fn compile_time_checked_query(session: &CachingSession) {
    // Generate a person, it can be asserted later that the persons are equal to the queried persons
    let person = Person {
        name: "jhon".to_string(),
        age: 20,
        email: "nothing special".to_string(),
        json_example: "not important".to_string(),
    };

    // Remember, you can not insert an owned struct, borrowed values only
    person.to_ref().insert(session).await.unwrap();

    let persons = query_persons_older_than(&person.name, person.age - 1).unwrap();

    // Since persons is of type SelectMultiple, functions are available to query multiple rows in the person table
    // Including paging, limiting and loading everything in memory
    // For now, just load everything into memory
    let persons = persons
        .select_all_in_memory(&session, 10)
        .await
        .unwrap()
        .entities;

    assert_eq!(1, persons.len());
    assert_eq!(person, persons[0]);
}

fn query_persons_older_than(
    name: &str,
    age: i32,
) -> Result<SelectMultiple<Person>, SerializeValuesError> {
    let result =
        catalytic_macro::query!("select * from person where name = ? and age > ?", name, age);

    Ok(result)
}

async fn create_session() -> CachingSession {
    // Make sure there is logging available when executing the statements
    tracing_subscriber::fmt::init();

    // Create a session:
    //      - which can operate on a single node
    //      - caches 1_000 queries in memory
    let session = CachingSession::from(
        SessionBuilder::new()
            .known_node(database_url())
            .default_consistency(Consistency::One)
            .build()
            .await
            .unwrap(),
        1_000,
    );

    // Use the keyspace
    session
        .session
        .use_keyspace("scylla_mapping", false)
        .await
        .unwrap();

    session
}

/// This is an example what you can do with a Person
/// You can only do CRUD operations with structs which borrows values, not owned struct
async fn crud_person(session: &CachingSession) -> Result<(), QueryError> {
    // This is an owned struct
    // You can convert this to a primary key or a borrowed version
    let person = Person {
        name: "Jeff".to_string(),
        age: 52,
        email: "hi_my_name_is_jeff@hotmail.com".to_string(),
        json_example: "something".to_string(),
    };

    // Insert the person
    // First convert it to the borrowed version
    person.to_ref().insert(session).await?;

    // Select the person back in memory
    // This will return an owned struct
    let person_queried = person
        .primary_key()
        .select_unique_expect(session)
        .await
        .unwrap()
        .entity;

    assert_eq!(person, person_queried);

    // Update the email column of person
    // Updating and deleting should always be executed on the borrowed version of the primary key
    // since you can only update/delete 1 row
    let pk = person.primary_key();

    pk.update_email(session, "new@email.com").await?;

    // Delete the row in the database
    pk.delete(session).await?;

    Ok(())
}
