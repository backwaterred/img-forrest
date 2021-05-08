
## Features

The main feature of this image server is the lazy (as in on-demand) populated, disk backed cache. When a user logs out images that are newly added, or updated since login will be written to the system disk. When a user asks for an image, the in-memory cache is checked. If the image is not in memory, it will be loaded from disk if present. The database can be queried to learn if an item is present without bringing it into memeory.

In addition to the disk-backed cache, the server provides logon/logoff, add/remove, and view capabilites. This is present mainly to showcase the disk backed cache. The logon/logoff functionality is especially trivial, and should not be expeced to hold up under any serious (cyber) attack. Add, remove, and view are expected to work well and showcase the functionality of the cache.

The server comes pre-loaded with several users who love nuts, hate cats, and are storing images in the cache. See the note in [logon](#logon) for more info.

## Docs

The following REST endpoints are offered.

### View

`GET /view/<image-id>`

Views an image on the server. Unless the image has been added with public set to true, *Login required*.

##### Example

Point a browser to [/view/out-on-the-town](http://localhost:8080/view/out-on-the-town)

#### Return Codes

- *200*: On success.
- *401*: When not logged in and the image is private.
- *404*: When the image cannot be found on the server.

### Add

`POST /add`

Adds an image to the server. *Login required*

Body must contain a JSON-object with *id* and *img*. The max size of the request is 1 MiB.

- *id*: String. Specifies the database-wide image id. Can be any valid unicode.
- *img*: String. The base64 encoded image data.
- *public*: Boolean (optional). Specifies whether the image is accessible by anyone, or just the user who uploaded it.

##### Example
```
{
    "id" : "bounty",
    "public" : false,
    "img" : "<base64-encoded-image-data>"
}
```

#### Return Codes

- *200*: On success.
- *401*: When not logged in.
- *409*: When image id cannot be added because it is already in use.

### Remove

`POST /remove`

Removes an image from the server. *Login required*

The body of the request must contain a JSON-object with the *id* of the image to be removed.

- *id*: String. Specifies the database-wide image id. Can be any valid unicode.

##### Example
```
{ 
    "id" : "a-normal-cat"
}
```

#### Return Codes

- *200*: On success.
- *401*: When not logged in.
- *404*: When image id cannot be removed because it cannot be found.

### Logon

`POST /logon`

Logs on to the server by setting the auth-user field in an encrypted cookie. This simulates a bearer token.

Body must contain a JSON-object with *uname* and *hpass*.

- *uname*: String. The username present in the table of users.
- *hpass*: String. The password hash present in the table of users.

> *Note:* The user table is initialized with users: chipper, nutty and blitz. Their password hashes are all 5f4dcc3b5aa765d61d8327deb882cf99.

##### Example
```
{ 
    "uname" : "blitz",
    "hpass" : "5f4dcc3b5aa765d61d8327deb882cf99"
}
```

#### Return Codes

- *200*: On success.
- *401*: When authentication is unsuccessful.

### Logoff

`POST /logoff`

Logs off the server.

No body is required. The server simply invalidates the current logon cookie if present.

#### Return Codes

- *200*: On success.


