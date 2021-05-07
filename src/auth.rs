use actix_web;
use actix_session::{
    Session
};
use crate::UserKey;

pub fn get_auth_user(sess: &Session) -> Option<UserKey>
{
    match sess.get("auth-user")
    {
        Ok(opt) =>
            opt,
        Err(_) =>
            None,
    }
}

pub fn authorize_user(sess: &mut Session, user: UserKey) -> actix_web::Result<()>
{
    sess.set("auth-user", user)
}

#[cfg(test)]
mod test
{
    #[ignore]
    #[test]
    fn auth_tests()
    {
        // I'd like to test *all* the things, but getting a valid Session object
        // has proven to be really challenging. So I'm leaving testing of these functions
        // out of scope for this project.
        unimplemented!()
    }
}
