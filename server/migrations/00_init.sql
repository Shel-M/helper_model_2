create table chores_persons (
	chore_id integer not null,
	person_id integer not null,

	primary key(chore_id, person_id)
);

create table chore (
	id integer not null,
	name char(50) not null,
	desc char(150),
	frequency integer,
	discord_channel char(18),

	primary key(id autoincrement)
);

create table person (
	id integer not null,
	name char(50) not null,
	discord_tag char(30),

	primary key(id autoincrement)
);

create table assignment (
	id integer not null, 
	chore_id integer not null,
	person_id integer not null,
	assignment_date datetime not null, -- Naive date, days since 1/1/1 CE via chrono::Utc::now().num_days_from_ce()
	reminder_date datetime, -- Naive date, see assignment_date
	completion_date datetime, -- Naive date, see assignment_date
	completed_person integer,

	primary key(id autoincrement)
);
