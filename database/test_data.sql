INSERT INTO users (email, username, password_hash) VALUES ('example@example.com', 'example', '');
INSERT INTO arguments (created_by, body) VALUES (1, 'Firearms should be legal to own and operate');
INSERT INTO arguments (created_by, body) VALUES (1, 'Firearms should be illegal to own and operate');
INSERT INTO arguments (parent, created_by, body) VALUES (1, 1, 'There is less gun crime in countries with more strict gun laws');
INSERT INTO arguments (parent, created_by, body) VALUES (2, 1, 'Every person has the right to protect themselves');
INSERT INTO topics (name, for_argument, against_argument) VALUES ('Should firearms be legal?', 1, 2);
