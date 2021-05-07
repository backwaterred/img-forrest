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
    fn auth_user_returns_some()
    {
        unimplemented!()
        // assert!(get_auth_user(&sess).is_some())
    }

    #[ignore]
    #[test]
    fn unauth_user_returns_none()
    {
        unimplemented!()
        // assert!(get_auth_user(&sess).is_none())
    }

    #[ignore]
    #[test]
    fn authorize_user()
    {
        unimplemented!()
        // authorize_user(&sess, user_key).unwrap();
        // assert_eq!(user_key, get_auth_user(&sess).unwrap())
    }
}
