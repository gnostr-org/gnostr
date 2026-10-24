#ifndef DAMUS_C_CONFIG_H
#define DAMUS_C_CONFIG_H

/* CCAN feature configuration for the Apple Clang targets in gnostr.xcodeproj. */
#if !defined(__APPLE__) || !defined(__clang__)
#error "damus-c/config.h requires Apple platforms and Clang"
#endif

#define HAVE_ALIGNOF 1
#define HAVE_TYPEOF 1
#define HAVE_STATEMENT_EXPR 1

#define HAVE_ATTRIBUTE_COLD __has_attribute(cold)
#define HAVE_ATTRIBUTE_CONST __has_attribute(const)
#define HAVE_ATTRIBUTE_DEPRECATED __has_attribute(deprecated)
#define HAVE_ATTRIBUTE_NONNULL __has_attribute(nonnull)
#define HAVE_ATTRIBUTE_NORETURN __has_attribute(noreturn)
#define HAVE_ATTRIBUTE_PRINTF __has_attribute(format)
#define HAVE_ATTRIBUTE_PURE __has_attribute(pure)
#define HAVE_ATTRIBUTE_RETURNS_NONNULL __has_attribute(returns_nonnull)
#define HAVE_ATTRIBUTE_SENTINEL __has_attribute(sentinel)
#define HAVE_ATTRIBUTE_UNUSED __has_attribute(unused)
#define HAVE_ATTRIBUTE_USED __has_attribute(used)
#define HAVE_WARN_UNUSED_RESULT __has_attribute(warn_unused_result)

#define HAVE_BUILTIN_CHOOSE_EXPR __has_builtin(__builtin_choose_expr)
#define HAVE_BUILTIN_CONSTANT_P __has_builtin(__builtin_constant_p)
#define HAVE_BUILTIN_CPU_SUPPORTS __has_builtin(__builtin_cpu_supports)
#define HAVE_BUILTIN_EXPECT __has_builtin(__builtin_expect)
#define HAVE_BUILTIN_TYPES_COMPATIBLE_P __has_builtin(__builtin_types_compatible_p)

#define HAVE_BYTESWAP_H 0
#define HAVE_BSWAP_64 0
#define HAVE_ISBLANK 1
#define HAVE_MEMMEM 1
#define HAVE_MEMRCHR 0
#define HAVE_VALGRIND_MEMCHECK_H 0

/* Keep SHA-256 loads aligned even on CPUs that support unaligned access. */
#define HAVE_UNALIGNED_ACCESS 0

#if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
#define HAVE_LITTLE_ENDIAN 1
#define HAVE_BIG_ENDIAN 0
#elif __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
#define HAVE_LITTLE_ENDIAN 0
#define HAVE_BIG_ENDIAN 1
#else
#error "Unsupported byte order"
#endif

#endif /* DAMUS_C_CONFIG_H */
