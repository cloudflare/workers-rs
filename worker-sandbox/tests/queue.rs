use retry::delay::Fixed;
use worker_sandbox::QueueBody;

use crate::util::{expect_wrangler, get, post};

mod util;
#[test]
fn send_message_to_queue() {
    // Arrange
    expect_wrangler();

    let id = "12345";

    // Act
    let response = post(&format!("queue/send/{id}"), |r| r);

    //Assert
    let status = response.status();
    assert!(status.is_success());
}

#[test]
fn receive_message_from_queue() {
    // Arrange
    expect_wrangler();

    let id = "12345";

    let send_message_response = post(&format!("queue/send/{id}"), |r| r);
    let send_message_status = send_message_response.status();
    assert!(send_message_status.is_success());

    // Act
    let message = retry::retry(Fixed::from_millis(500).take(5), || {
        let messages: Vec<QueueBody> = get("queue", |r| r).json().expect("Failed to get Json");

        match messages.iter().find(|m| m.id == id) {
            Some(m) => Ok(m.clone()),
            None => Err("Failed to find expected message"),
        }
    })
    .unwrap();

    // Assert
    assert_eq!(message.id, id);
}
