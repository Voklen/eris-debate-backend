When creating an account the user's password will be sent over https to the server, where it is then hashed by Argon2id with a salt and pepper.

$$ Hashed Password = \operatorname{Argon2id}(Password, Salt, Pepper) $$

# Reasons
- The hash is so that if the database is exposed, the hashes cannot be used to log in.
- The salt is to prevent a rainbow table
- The pepper to prevent easy passwords from being cracked if only the database is leaked

## Why Argon2id?
According to [OWASP](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id)

> Argon2 was the winner of the 2015 Password Hashing Competition. Out of the three Argon2 versions, use the Argon2id variant since it provides a balanced approach to resisting both side-channel and GPU-based attacks.

## Client side hashing
I was considering hashing client side as well, but after review decided against it because

1. It was not intended to be hashed twice, and this can lead to [certain vulnerabilities](https://blog.ircmaxell.com/2015/03/security-issue-combining-bcrypt-with.html)
2. It is standard practice to send passwords over https and then hash them server-side
3. It adds more complexity (which could manifest in things like bugs in the JS hashing implementation that result in many collisions)
4. It adds additional load on the server because they have to fetch the javascript hash implementation (and it would have to be on our servers to not increase the attack area)
5. This is effectively trying to roll our own cryptography, _which you should never do_
