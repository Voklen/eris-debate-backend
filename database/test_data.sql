INSERT INTO users (email, username, password_hash, email_verified) VALUES ('example@example.com', 'example', '$argon2id$v=19$m=19456,t=2,p=1$ilQT9xqW8Ozg9+Zs98EjhA$kGx50GjIgMbvmYGFo6NPU5R4/hMuu5W0C2Bh4KD1sNQ', true);
INSERT INTO revisions (revision_by, body) VALUES (1, 'Firearms should be legal to own and operate');
INSERT INTO arguments (revision_latest) VALUES (1);
INSERT INTO revisions (revision_by, body) VALUES (1, 'Firearms should be illegal to own and operate');
INSERT INTO arguments (revision_latest) VALUES (2);
INSERT INTO revisions (revision_by, body) VALUES (1, 'There is less gun crime in countries with more strict gun laws');
INSERT INTO arguments (revision_latest, parent) VALUES (3, 1);
INSERT INTO revisions (revision_by, body) VALUES (1, 'Every person has the right to protect themselves');
INSERT INTO arguments (revision_latest, parent) VALUES (4, 2);
INSERT INTO topics (name, for_argument, against_argument) VALUES ('Should firearms be legal?', 1, 2);
