#include <sqlite3.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static char *USAGE_FMT = "Usage: %s DB_FILE\n"
                         "   DB_FILE always gets overwritten with a database "
                         "with basic 'Sample' table.\n"
                         "\n"
                         "Program exits on any system error!\n"
                         "\n";

int insert_sample_data(char *db_file);
int read_sample_data(char *db_file);

int file_exists(char *file_path) {
  struct stat buff;
  return (stat(file_path, &buff) == 0);
}

int main(int argc, char **argv) {
  if (argc != 2 || 0 == strcmp(argv[1], "--help") ||
      0 == strcmp(argv[1], "-h")) {
    fprintf(stderr, USAGE_FMT, argv[0], argv[0]);
    return -1;
  }

  printf("1. Running with SQLite version %s\n", sqlite3_libversion());

  char *db_file = argv[1];
  printf("2. Using db file %s\n", db_file);
  if (file_exists(db_file)) {
    printf("File '%s' exists. Removing...\n", db_file);
    int remove_result = remove(db_file);
    if (remove_result) {
      perror(db_file);
      return -1;
    }
  }

  printf("3. Creating 'Sample' table data...\n");
  if (insert_sample_data(db_file) != 0) {
    fprintf(stderr, "Failed to create 'Sample' table data!\n");
    return -1;
  }

  /*
printf("4. Reading 'Sample' table data...\n");
if (read_sample_data(db_file) != 0) {
  fprintf(stderr, "Failed to read 'Sample' table data!\n");
  return -1;
}
*/

  return 0;
}

int insert_sample_data(char *db_file) {
  sqlite3 *db;
  if (sqlite3_open(db_file, &db) != SQLITE_OK) {
    fprintf(stderr, "Failed to open db in '%s' with error: %s\n", db_file,
            sqlite3_errmsg(db));
    sqlite3_close(db);
    return -1;
  }

  char *sql_command = "DROP TABLE IF EXISTS Sample;"
                      "CREATE TABLE Sample ("
                      "    id INTEGER PRIMARY KEY,"
                      "    random_value INTEGER"
                      ");";

  char *error_message = NULL;
  if (sqlite3_exec(db, sql_command, 0, 0, &error_message) != SQLITE_OK) {
    fprintf(stderr, "SQL execution failed with error: %s\n", error_message);

    sqlite3_free(error_message);
    sqlite3_close(db);

    return -1;
  }

  for (int i = 0; i < 1000; ++i) {
    char insert_command[100];
    int rand_int = rand() % 100;
    sprintf(insert_command, "INSERT INTO Sample (random_value) VALUES (%d);",
            rand_int);
    if (sqlite3_exec(db, insert_command, 0, 0, &error_message) != SQLITE_OK) {
      fprintf(stderr, "SQL execution failed with error: %s\n", error_message);

      sqlite3_free(error_message);
      sqlite3_close(db);

      return -1;
    }
  }

  sqlite3_close(db);
  return 0;
}

// Note: While simple to read, the callback approach is deprecated!
int read_lines_cb(void *linePtr, int columns, char **values, char **names) {
  int *line = (int *)linePtr;
  // Print headers on first line
  if (*line == 0) {
    for (int i = 0; i < columns; ++i)
      printf("|%-20s", names[i]);

    printf("|\n");

    for (int i = 0; i < columns; ++i)
      printf("+====================");

    printf("+\n");
  }

  (*line)++;

  // Print values
  for (int i = 0; i < columns; ++i)
    printf("|%-20s", values[i] ? values[i] : "NULL");

  printf("|\n");
  return 0;
}

int read_sample_data(char *db_file) {
  sqlite3 *db;
  if (sqlite3_open(db_file, &db) != SQLITE_OK) {
    fprintf(stderr, "Failed to open db in '%s' with error: %s\n", db_file,
            sqlite3_errmsg(db));
    sqlite3_close(db);
    return -1;
  }

  char *sql_command = "SELECT * FROM Sample";
  char *error_message = NULL;
  int line_number = 0;
  if (sqlite3_exec(db, sql_command, read_lines_cb, &line_number,
                   &error_message) != SQLITE_OK) {
    fprintf(stderr, "SQL execution failed with error: %s\n", error_message);

    sqlite3_free(error_message);
    sqlite3_close(db);

    return -1;
  }
  printf("%d total record(s).\n", line_number);

  sqlite3_close(db);
  return 0;
}