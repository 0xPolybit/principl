/// Minimal C runtime support used by the v0.1 Windows bootstrap backend.
pub(crate) const C_RUNTIME: &str = r#"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void princi_print_int(int64_t value) {
    printf("%lld\n", (long long)value);
}

static void princi_print_float(double value) {
    printf("%.15g\n", value);
}

static void princi_print_bool(bool value) {
    puts(value ? "true" : "false");
}

static void princi_print_string(const char *value) {
    printf("%s\n", value);
}

static const char *princi_concat(const char *left, const char *right) {
    size_t left_length = strlen(left);
    size_t right_length = strlen(right);
    if (right_length == SIZE_MAX || left_length > SIZE_MAX - right_length - 1) {
        fputs("princi: string is too large\n", stderr);
        exit(EXIT_FAILURE);
    }
    char *result = (char *)malloc(left_length + right_length + 1);
    if (result == NULL) {
        fputs("princi: could not allocate string\n", stderr);
        exit(EXIT_FAILURE);
    }
    memcpy(result, left, left_length);
    memcpy(result + left_length, right, right_length + 1);
    return result;
}
"#;
