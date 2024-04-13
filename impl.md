# Implementation remarks:
## REDIS
In order to cross-reference ACCESS and REFRESH tokens, the reference is part of the keyed value. With this way of implementation it is possible to invalidate an ACCESS token with a given REFRESH token. Usually its not needed to do it the other way around (getting the REFRESH token with the ACCESS token) but for now its available like that. (Mental Note: might be a security risk for the later way)
### AccessTokens
`ACCESS:{{UUID}} => USER_ID:REFRESH_TOKEN`  

### RefreshTokens
`REFRESH:{{UUID}} => USER_ID:ACCESS_TOKEN`

## query-files (queries.rs)
All actions that execute a query shall use a prefix to indicate the type of operation:  
* `i` indicates insertions  
* `q` indicate queries  
* `d` indicate deletions  
* `u` indicate updates

An example: `i_otp` tells us, that we are going to insert an OTP (One-Time-Password) into the respective database and table
