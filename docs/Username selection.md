
# Problem
The way we prevent mass creation of accounts is by requiring a verified email. The problem is that if the user enters a username on signup, that username is put into the `users` table which has `UNIQUE username`.
This means that if a user
1. Create account with mistyped email
2. Corrects email and creates account again
3. Username already in use

# Solutions
## Remove `UNIQUE` from `username`
This would solve the issue but it would mean that I can’t guarantee there is only one username. For username login this would make it quite inconvenient so I’d prefer to avoid this option.

## Ask for username after email verification
If I make `username` nullable this could work and the benefit would be to mean there’s less of a barrier for initial signup. I don’t like this though because it means there’s an extra hidden step after you feel like you can start using this and also people don’t feel like they can choose their username at first. It would also mean that if the user closes the tab to choose a username, every time they login there would need to be a check that could redirect them to the choose a username page before they can do anything which means quite a bit more redirection code to maintain.

## `temp_username` column
Create a non-`UNIQUE`column called `temp_username` in the users table, and then add the username to that and only put it into `username` when the user has be verified. This is an option I’d be more happy with.

## Unverified user
Create a `unverified_users` table, and then when a user confirms the email move them into `users`. I think this is the best option because it’s more normalised than the [[#`temp_username` column | temp_username column]] and just feels more logical (to me at least).

# Decision
I've decided to make a `unverified_users` table and then copy the data over on email verification
