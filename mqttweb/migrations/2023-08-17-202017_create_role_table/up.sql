CREATE TABLE "roles" (
	"id"	INTEGER NOT NULL UNIQUE,
	"name" TEXT NOT NULL UNIQUE,
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE "user_role" (
	"user_id"	INTEGER NOT NULL,
	"role_id"	INTEGER NOT NULL,
  PRIMARY KEY("user_id", "role_id")
	FOREIGN KEY("user_id") REFERENCES "users"("id"),
	FOREIGN KEY("role_id") REFERENCES "roles"("id")
);