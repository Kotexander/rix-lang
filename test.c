int printf(const char *, ...);
int square(int);

int main(int argc, const char **argv) {
  if (argc > 1) {
    printf("Multiple arguments provided. First argument: %s", argv[0]);
  } else if (argc == 1) {
    printf("One argument provided.");
  } else {
    printf("No arguments provided.");
  }

  int i = 0;
  while (i < argc) {
    printf("Argument %d: %s", i, argv[i]);
    i = i + 1;
  }
  printf("Square of argc: %d\n", square(argc));

  return square(0);
}

int square(int num) { return num * num; }
