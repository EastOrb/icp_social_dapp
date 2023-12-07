#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Message {
    id: u64,
    title: String,
    body: String,
    attachment_url: String,
    created_at: u64,
    updated_at: Option<u64>,
    upvotes: u64,
    downvotes: u64,
    upvoted_users: Vec<String>,
    downvoted_users: Vec<String>,
}
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Comment {
    id: u64,
    message_id: u64,
    user: String,
    content: String,
    created_at: u64,
}

impl Storable for Comment {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Comment {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType,Clone, Serialize, Deserialize, Default)]
struct Report {
    id: u64,
    message_id: u64,
    reported_by: String,
    reason: String,
    reported_at: u64,
    reviewed: bool,
}

impl Storable for Report {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Report {
    const MAX_SIZE: u32 = 2048;
    const IS_FIXED_SIZE: bool = false;
}



// Implement Storable and BoundedStorable for Message
impl Storable for Message {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Message {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct MessagePayload {
    title: String,
    body: String,
    attachment_url: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct User {
    username: String,
    tokens: u64,
}

impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024; // Set an appropriate max size
    const IS_FIXED_SIZE: bool = false;
}
#[derive(candid::CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UserId(u64);

impl Storable for UserId {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&self.0).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        UserId(Decode!(bytes.as_ref(), u64).unwrap())
    }
}

impl BoundedStorable for UserId {
    const MAX_SIZE: u32 = std::mem::size_of::<u64>() as u32;
    const IS_FIXED_SIZE: bool = true;
}



thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, Message, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        ));

    static USERS: RefCell<StableBTreeMap<UserId, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        ));
    static COMMENTS: RefCell<StableBTreeMap<u64, Comment, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))))
    );
    static REPORTS: RefCell<StableBTreeMap<u64, Report, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
    );
}


#[ic_cdk::query]
fn get_message(id: u64) -> Result<Message, Error> {
    match _get_message(&id) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!("A message with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn add_message(message: MessagePayload) -> Option<Message> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");
    let message = Message {
        id,
        title: message.title,
        body: message.body,
        attachment_url: message.attachment_url,
        created_at: time(),
        updated_at: None,
        upvotes: 0,
        downvotes: 0,
        upvoted_users: Vec::new(),
        downvoted_users: Vec::new(),
    };
    do_insert(&message);
    Some(message)
}

#[ic_cdk::update]
fn update_message(id: u64, payload: MessagePayload) -> Result<Message, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut message) => {
            message.attachment_url = payload.attachment_url;
            message.body = payload.body;
            message.title = payload.title;
            message.updated_at = Some(time());
            do_insert(&message);
            Ok(message)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a message with id={}. Message not found", id),
        }),
    }
}

// Helper method to perform insert.
fn do_insert(message: &Message) {
    STORAGE.with(|service| service.borrow_mut().insert(message.id, message.clone()));
}

#[ic_cdk::update]
fn delete_message(id: u64) -> Result<Message, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a message with id={}. Message not found.", id),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    AlreadyVoted { msg: String },
    UserNotFound { msg: String },
}

// A helper method to get a message by id. Used in get_message/update_message
fn _get_message(id: &u64) -> Option<Message> {
    STORAGE.with(|service| service.borrow().get(id))
}

#[ic_cdk::update]
fn upvote_message(id: u64, username: String) -> Result<(), Error> {
    let (mut message, found) = STORAGE.with(|s| {
        let storage = s.borrow_mut();
        // Immediately retrieve and clone the message, then drop the mutable borrow
        match storage.get(&id) {
            Some(message) => (message.clone(), true),
            None => (Message::default(), false),
        }
    });

    // Check if the message was found
    if !found {
        return Err(Error::NotFound {
            msg: format!("Message with id={} not found", id),
        });
    }

    // Proceed with the rest of the function
    if !message.upvoted_users.contains(&username) {
        message.upvotes += 1;
        message.upvoted_users.push(username.clone());

        // Re-insert the modified message
        STORAGE.with(|s| s.borrow_mut().insert(id, message));

        // Assuming reward_upvote works with a String
        reward_upvote(username)?;
        Ok(())
    } else {
        Err(Error::AlreadyVoted {
            msg: "User has already upvoted this message".to_string(),
        })
    }
}


        
#[ic_cdk::update]
fn downvote_message(id: u64, username: String) -> Result<(), Error> {
    let (mut message, found) = STORAGE.with(|s| {
        let storage = s.borrow_mut();
        // Immediately retrieve and clone the message, then drop the mutable borrow
        match storage.get(&id) {
            Some(message) => (message.clone(), true),
            None => (Message::default(), false),
        }
    });

    // Check if the message was found
    if !found {
        return Err(Error::NotFound {
            msg: format!("Message with id={} not found", id),
        });
    }

    // Proceed with the rest of the function
    if !message.downvoted_users.contains(&username) {
        message.downvotes += 1;
        message.downvoted_users.push(username.clone());

        // Re-insert the modified message
        STORAGE.with(|s| s.borrow_mut().insert(id, message));

        Ok(())
    } else {
        Err(Error::AlreadyVoted {
            msg: "User has already downvoted this message".to_string(),
        })
    }
}


        
#[ic_cdk::update]
fn reward_upvote(username: String) -> Result<(), Error> {
    // Find the UserId associated with the given username and update the user in the same scope
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let user_id = users.iter().find_map(|(id, user)| {
            if user.username == username {
                Some(id) // Get the UserId
            } else {
                None
            }
        });

        if let Some(user_id) = user_id {
            if let Some(user) = users.get(&user_id) {
                // Update tokens without cloning
                let updated_user = User {
                    username: user.username.clone(), // Assuming String fields are clonable
                    tokens: user.tokens + 1,
                    // copy other fields as needed...
                };

                // Re-insert the updated user
                users.insert(user_id, updated_user);
                Ok(())
            } else {
                Err(Error::UserNotFound {
                    msg: format!("User {} not found", username),
                })
            }
        } else {
            Err(Error::UserNotFound {
                msg: format!("User {} not found", username),
            })
        }
    })
}
#[ic_cdk::query]
fn search_messages(search_term: Option<String>, min_upvotes: Option<u64>, max_downvotes: Option<u64>, recent: Option<u64>) -> Vec<Message> {
    let now = time();
    STORAGE.with(|s| {
        s.borrow()
         .iter()
         .filter(|(_, message)| {
             let matches_term = search_term.as_ref().map_or(true, |term| {
                 message.title.contains(term) || message.body.contains(term)
             });
             let matches_upvotes = min_upvotes.map_or(true, |min| message.upvotes >= min);
             let matches_downvotes = max_downvotes.map_or(true, |max| message.downvotes <= max);
             let matches_recent = recent.map_or(true, |hours| now - message.created_at <= hours * 3600);

             matches_term && matches_upvotes && matches_downvotes && matches_recent
         })
         .map(|(_, message)| message.clone())
         .collect()
    })
}


#[ic_cdk::update]
fn add_comment(message_id: u64, user: String, content: String) -> Result<Comment, Error> {
    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        let _ = counter.borrow_mut().set(current_value + 1);
        current_value + 1
    });

    let comment = Comment {
        id,
        message_id,
        user,
        content,
        created_at: time(),
    };
    COMMENTS.with(|c| c.borrow_mut().insert(id, comment.clone()));
    Ok(comment)
}


#[ic_cdk::query]
fn get_comments(message_id: u64) -> Vec<Comment> {
    COMMENTS.with(|c| {
        c.borrow()
         .iter()
         .filter(|(_, comment)| comment.message_id == message_id)
         .map(|(_, comment)| comment.clone())
         .collect()
    })
}


#[ic_cdk::update]
fn delete_comment(comment_id: u64) -> Result<(), Error> {
    match COMMENTS.with(|c| c.borrow_mut().remove(&comment_id)) {
        Some(_) => Ok(()),
        None => Err(Error::NotFound {
            msg: format!("Comment with id={} not found.", comment_id),
        }),
    }
}
#[ic_cdk::update]
fn report_message(message_id: u64, reported_by: String, reason: String) -> Result<Report, Error> {
    let report_id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        let _ = counter.borrow_mut().set(current_value + 1);
        current_value + 1
    });

    let report = Report {
        id: report_id,
        message_id,
        reported_by,
        reason,
        reported_at: time(),
        reviewed: false,
    };

    REPORTS.with(|r| r.borrow_mut().insert(report_id, report.clone()));
    Ok(report)
}

#[ic_cdk::update]
fn review_report(report_id: u64, _action: String) -> Result<(), Error> {
    REPORTS.with(|r| {
        let mut reports = r.borrow_mut();
        if let Some(mut report) = reports.remove(&report_id) {
            report.reviewed = true;
            // Perform action based on the provided string, e.g., delete message, issue warning, etc.

            // Re-insert the updated report
            reports.insert(report_id, report);
            Ok(())
        } else {
            Err(Error::NotFound {
                msg: format!("Report with id={} not found.", report_id),
            })
        }
    })
}






    








        
// Need this to generate candid
ic_cdk::export_candid!();