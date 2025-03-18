use sqlite_dbhash::Selection;

use crate::harness::{Step, run_tests};

mod harness;

#[test]
pub fn test_multiple_tables_normal() {
    run_tests(
        "test_multiple_tables_normal",
        [
            Step::sql(
                "
                CREATE TABLE prefix1_t1
                (
                    intval INT,
                    textval TEXT
                );
                ",
            ),
            Step::sql(
                "
                CREATE TABLE prefix1_t2
                (
                    intval1 INT NOT NULL,
                    intval2 INT,
                    intval3 INT
                );
                ",
            ),
            Step::sql(
                "
                CREATE TABLE prefix2_t1
                (
                    textval1 TEXT,
                    textval2 TEXT,
                    textval3 TEXT UNIQUE NOT NULL
                );
                ",
            ),
            Step::sql(
                "
                CREATE TABLE prefix2_t2
                (
                    intval INT,
                    textval TEXT
                );
                ",
            ),
            Step::sql(
                "
                INSERT INTO prefix1_t1
                VALUES
                    (0, 'a');
                ",
            ),
            Step::sql(
                "
                INSERT INTO prefix1_t2
                VALUES
                    (0, 1, 2);
                ",
            ),
            Step::sql(
                "
                INSERT INTO prefix2_t1
                VALUES
                    ('a', 'b', 'c');
                ",
            ),
            Step::sql(
                "
                INSERT INTO prefix2_t2
                VALUES
                    (1, '');
                ",
            ),
            Step::compare(Some("prefix%"), Selection::SchemaOnly),
            Step::compare(Some("prefix1%"), Selection::SchemaAndContent),
            Step::compare(Some("prefix2%"), Selection::ContentOnly),
            Step::compare(Some("%t1"), Selection::SchemaAndContent),
            Step::compare(Some("%t2"), Selection::SchemaAndContent),
            Step::compare(Some("%t%"), Selection::SchemaAndContent),
            Step::sql(
                "
                INSERT INTO prefix1_t1
                VALUES
                    (1, 'a'),
                    (2, 'b'),
                    (3, 'c');
                INSERT INTO prefix1_t2
                VALUES
                    (1, 10, 100),
                    (2, 20, 400),
                    (3, 30, 900);
                INSERT INTO prefix2_t1
                VALUES
                    ('', NULL, ''),
                    ('💖', 'y̆', 'β'),
                    ('Löwe', '老虎', 'Léopard');
                INSERT INTO prefix2_t2
                VALUES
                    (-1, 'a'),
                    (-2, 'b'),
                    (-3, 'c');
                ",
            ),
            Step::compare(Some("prefix%"), Selection::SchemaOnly),
            Step::compare(Some("prefix1%"), Selection::SchemaAndContent),
            Step::compare(Some("prefix2%"), Selection::ContentOnly),
            Step::compare(Some("%t1"), Selection::SchemaAndContent),
            Step::compare(Some("%t2"), Selection::SchemaAndContent),
            Step::compare(Some("%t%"), Selection::SchemaAndContent),
            Step::sql(
                "
                CREATE UNIQUE INDEX idx_p1t1 ON prefix1_t1 (intval);
                CREATE INDEX idx_p1t2 ON prefix1_t2 (intval2, intval1, intval3);
                CREATE UNIQUE INDEX str_p2t1 ON prefix2_t1 (textval3, textval1);
                ",
            ),
            Step::compare(Some("prefix%"), Selection::SchemaOnly),
            Step::compare(Some("prefix1%"), Selection::SchemaAndContent),
            Step::compare(Some("%t1"), Selection::SchemaAndContent),
            Step::compare(Some("%t2"), Selection::SchemaAndContent),
            Step::compare(Some("%t%"), Selection::SchemaAndContent),
        ],
    );
}
