// vim: set foldmethod=marker:

#include <stdlib.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h> // used for uint64_t/uintptr_t/int64_t/intptr_t
#include <stdio.h> // used for printf

#include <time.h> // used for varela's logger

#ifdef _WIN32
    #include <windows.h> // used for terminal colors
#endif

// pa-mims/my-static-assert - Copyright (c) 2021 pa-mims - The Unlicense
// {{{

#if defined( __STDC_VERSION__ ) && __STDC_VERSION__ >= 201112L
    #define STATIC_ASSERT(cond, msg) _Static_assert(cond, msg)
#else // We make our own
    // JOIN macro tricks the preprocessor into generating a unique token
    #define STX_JOIN2(pre, post) STX_JOIN3(pre, post)
    #define STX_JOIN3(pre, post) pre ## post

    #if defined( __COUNTER__ ) // try to do it the smart way...
        #define STX_JOIN(pre) STX_JOIN2(pre, __COUNTER__)
        #define STATIC_ASSERT(cond, msg) \
            static const char *STX_JOIN(static_assert)[(cond) * 2 - 1] = { msg }
    #else // we did our best...
        //will break if static assert is on same line in different files
        #define STX_JOIN(pre) STX_JOIN2(pre, __LINE__)
        #define STATIC_ASSERT(cond, msg) \
            static const char *STX_JOIN(static_assert)[(cond) * 2 - 1] = { msg }
    #endif
#endif

// }}}

// hazelutf8/c99_rust_types - Copyright (c) 2021 hazelutf8 - Apache/MIT License
// {{{

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
#ifdef UINT64_MAX
    typedef uint64_t u64;
#endif
#ifdef UINTPTR_MAX
    typedef uintptr_t usize;
#else
    typedef unsigned int usize;
#endif

typedef int8_t i8;
typedef int16_t i16;
typedef int32_t i32;
#ifdef INT64_MAX
    typedef int64_t i64;
#endif
#ifdef INTPTR_MAX
    typedef intptr_t isize;
#else
    typedef int isize;
#endif

typedef float f32;
typedef double f64;

#ifdef STATIC_ASSERT
    STATIC_ASSERT(sizeof(u8) == 1, "unsigned 1 byte");
    STATIC_ASSERT(sizeof(u16) == 2, "unsigned 2 bytes");
    STATIC_ASSERT(sizeof(u32) == 4, "unsigned 4 bytes");
    #ifdef UINT64_MAX
        STATIC_ASSERT(sizeof(u64) == 8, "unsigned 8 bytes");
    #endif

    STATIC_ASSERT(sizeof(i8) == 1, "signed 1 byte");
    STATIC_ASSERT(sizeof(i16) == 2, "signed 2 bytes");
    STATIC_ASSERT(sizeof(i32) == 4, "signed 4 bytes");
    #ifdef INT64_MAX
        STATIC_ASSERT(sizeof(i64) == 8, "signed 8 bytes");
    #endif
#endif

// }}}

// Txuritan/option - Copyright (c) Txuritan 2021 - MIT License
// {{{

struct Option { void * ptr; };

#define OPTION(type) \
    Option_##type

#define OPTION_IMPL(type) \
    typedef struct Option OPTION(type);

#define OPTION_SOME(value) \
    { .ptr = (void *) (value) }
#define OPTION_NONE() \
    { .ptr = (void *) NULL }

#define OPTION_IS_NONE(opt) \
    (((opt)->ptr) == NULL)
#define OPTION_IS_SOME(opt) \
    (!OPTION_IS_NONE(opt))

#define OPTION_UNWRAP(type, opt, value) \
    OPTION_IS_SOME(opt) ? ( *(value) = *((type *) (opt)->ptr) , true ) : (false)

typedef struct Option Option_Str;

OPTION_IMPL(bool);

OPTION_IMPL(u8);
OPTION_IMPL(u16);
OPTION_IMPL(u32);
OPTION_IMPL(u64);
OPTION_IMPL(usize);

OPTION_IMPL(i8);
OPTION_IMPL(i16);
OPTION_IMPL(i32);
OPTION_IMPL(i64);
OPTION_IMPL(isize);

OPTION_IMPL(f32);
OPTION_IMPL(f64);

// }}}

// Txuritan/byopts - Copyright (c) 2021 Txuritan - MIT License
// {{{

// TODO: add a way to check for and to convert a number into little endian

// unsigned integer operations
// {{{

void u8_to_bytes(u8 num, u8 * array) {}

void u8_from_bytes(const u8 * array, u8 * num) {}

void u16_to_bytes(u16 num, u8 * array) {
    array[0] = (u8)(num >> 0);
    array[1] = (u8)(num >> 8);
}

void u16_from_bytes(const u8 * array, u16 * num) {
    *num =
        (u16)array[0] << 0 |
        (u16)array[1] << 8;
}

void u32_to_bytes(u32 num, u8 * array) {
    array[0] = (u8)(num >>  0);
    array[1] = (u8)(num >>  8);
    array[2] = (u8)(num >> 16);
    array[3] = (u8)(num >> 24);
}

void u32_from_bytes(const u8 * array, u32 * num) {
    *num =
        (u32)array[0] <<  0 |
        (u32)array[1] <<  8 |
        (u32)array[2] << 16 |
        (u32)array[3] << 24;
}

#ifdef UINT64_MAX
void u64_to_bytes(u64 num, u8 * array) {}

void u64_from_bytes(const u8 * array, u64 * num) {}
#endif

// }}}

// signed integer operations
// {{{

void i8_to_bytes(i8 num, u8 * array) {}

void i8_from_bytes(const u8 * array, i8 * num) {}

void i16_to_bytes(i16 num, u8 * array) {}

void i16_from_bytes(const u8 * array, i16 * num) {}

void i32_to_bytes(i32 num, u8 * array) {}

void i32_from_bytes(const u8 * array, i32 * num) {}

#ifdef INT64_MAX
void i64_to_bytes(i64 num, u8 * array) {}

void i64_from_bytes(const u8 * array, i64 * num) {}
#endif

// }}}

// }}}

// Txuritan/aloene - Copyright (c) 2021 Txuritan - MIT License
// {{{

#define ALOENE_CONTAINER_UNIT (u8)0 // 0x00
#define ALOENE_CONTAINER_NONE (u8)1 // 0x01
#define ALOENE_CONTAINER_SOME (u8)2 // 0x02
#define ALOENE_CONTAINER_VALUE (u8)3 // 0x03
#define ALOENE_CONTAINER_VARIANT (u8)4 // 0x04
#define ALOENE_CONTAINER_STRUCT (u8)5 // 0x05
#define ALOENE_CONTAINER_ARRAY (u8)6 // 0x06
#define ALOENE_CONTAINER_MAP (u8)7 // 0x07
#define ALOENE_CONTAINER_LIST (u8)8 // 0x08

#define ALOENE_VALUE_BOOL (u8)0 // 0x00
#define ALOENE_VALUE_STRING (u8)1 // 0x01
#define ALOENE_VALUE_FLOAT_32 (u8)16 // 0x10
#define ALOENE_VALUE_FLOAT_64 (u8)17 // 0x11
#define ALOENE_VALUE_SIGNED_8(u8)32 // 0x20
#define ALOENE_VALUE_SIGNED_16 (u8)33 // 0x21
#define ALOENE_VALUE_SIGNED_32 (u8)34 // 0x22
#define ALOENE_VALUE_SIGNED_64 (u8)35 // 0x23
#define ALOENE_VALUE_SIGNED_SIZE (u8)36 // 0x24
#define ALOENE_VALUE_UNSIGNED_8(u8)48 // 0x30
#define ALOENE_VALUE_UNSIGNED_16 (u8)49 // 0x31
#define ALOENE_VALUE_UNSIGNED_32 (u8)50 // 0x32
#define ALOENE_VALUE_UNSIGNED_64 (u8)51 // 0x33
#define ALOENE_VALUE_UNSIGNED_SIZE (u8)52 // 0x34

enum AloeneErrorKind {
    AloeneErrorKind_InvalidByte,
};

struct AloeneError {
    enum AloeneErrorKind kind;
    char * message;
};

void aloene_string_read() {}

void aloene_string_write() {}

// }}}

// Txuritan/fenn - Copyright (c) 2021 Txuritan - MIT License
// {{{

// dynamic vector
// {{{

#define VEC(T) struct { T * ptr; u64 capacity; u64 length; }

#define VEC__UNPACK(v) \
    (char**)&(v)->ptr, &(v)->capacity, &(v)->length, sizeof(*(v)->data)

typedef VEC(char*) Vec_Str;

// }}}

// }}}

// Txuritan/varela - Copyright (c) 2021 - MIT License
// {{{

// TODO: figure out a good way to handle 'namespaces'

#define PASTE(x, y, z) x ## _ ## y ## _ ## z
#define EVAULATE(x, y, z) PASTE(x, y, z)
#define NAME(name) EVAULATE(LIBRARY, MODULE, name)

// varela-common
// {{{

struct VarelaCommon_Range {
    u64 start;
    u64 end;
};

typedef struct Option Option_VarelaCommon_Range;

enum VarelaCommon_Color {
    VarelaCommon_Color_Reset = 0,

    VarelaCommon_Color_Red = 1,
    VarelaCommon_Color_Green,
    VarelaCommon_Color_Yellow,
    VarelaCommon_Color_Blue,
    VarelaCommon_Color_Magenta,
    VarelaCommon_Color_Cyan,

    VarelaCommon_Color_BrightRed = 61,
    VarelaCommon_Color_BrightGreen,
    VarelaCommon_Color_BrightYellow,
    VarelaCommon_Color_BrightBlue,
    VarelaCommon_Color_BrightMagenta,
    VarelaCommon_Color_BrightCyan,
};

int VarelaCommon__colorf(enum VarelaCommon_Color color, const char * fmt) {
    int foreground_color = -1;
    int return_value = -99;

#ifdef _WIN32
    // TODO: this doesnt seem right, there must be a better way to do this
    switch (color) {
        case VarelaCommon_Color_Red: foreground_color = FOREGROUND_RED; break;
        case VarelaCommon_Color_Green: foreground_color = FOREGROUND_GREEN; break;
        case VarelaCommon_Color_Yellow: foreground_color = FOREGROUND_RED | FOREGROUND_GREEN; break;
        case VarelaCommon_Color_Blue: foreground_color = FOREGROUND_BLUE; break;
        case VarelaCommon_Color_Magenta: foreground_color = FOREGROUND_RED | FOREGROUND_BLUE; break;
        case VarelaCommon_Color_Cyan: foreground_color = FOREGROUND_GREEN | FOREGROUND_BLUE; break;

        case VarelaCommon_Color_BrightRed: foreground_color = FOREGROUND_INTENSITY | FOREGROUND_RED; break;
        case VarelaCommon_Color_BrightGreen: foreground_color = FOREGROUND_INTENSITY | FOREGROUND_GREEN; break;
        case VarelaCommon_Color_BrightYellow: foreground_color = FOREGROUND_INTENSITY | FOREGROUND_RED | FOREGROUND_GREEN; break;
        case VarelaCommon_Color_BrightBlue: foreground_color = FOREGROUND_INTENSITY | FOREGROUND_BLUE; break;
        case VarelaCommon_Color_BrightMagenta: foreground_color = FOREGROUND_INTENSITY | FOREGROUND_RED | FOREGROUND_BLUE; break;
        case VarelaCommon_Color_BrightCyan: foreground_color = FOREGROUND_INTENSITY | FOREGROUND_GREEN | FOREGROUND_BLUE; break;

        default: break;
    }

    CONSOLE_SCREEN_BUFFER_INFO console_buffer_info;
    WORD old_color_flags;
    HANDLE stdout_handle;

    if (color != VarelaCommon_Color_Reset) {
        stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if ((stdout_handle == NULL) || (stdout_handle == INVALID_HANDLE_VALUE)) {
            return -99;
        }

        if (GetConsoleScreenBufferInfo(stdout_handle, &console_buffer_info) == 0) {
            return -99;
        }

        old_color_flags = console_buffer_info.wAttributes;

        if (SetConsoleTextAttribute(stdout_handle, foreground_color) == 0) {
            return -99;
        }
    }
#else
    if (color != VarelaCommon_Color_Reset) {
        foreground_color = color + 30;
    }

    if (color != VarelaCommon_Color_Reset && printf("\033[%dm", foreground_color) < 0) {
        return -99;
    }
#endif

    return_value = printf("%s", fmt);

    if (foreground_color != -1) {
#ifdef _WIN32
        if (SetConsoleTextAttribute(stdout_handle, old_color_flags) == 0) {
            return -99;
        }
#else
        if (printf("\033[0m") < 0) {
            return -99;
        }
#endif
    }

    return return_value;
}

// varela-common :: logging
// {{{

enum VarelaCommon_Logger_LogLevel {
    VarelaCommon_Logger_LogLevel_Trace,
    VarelaCommon_Logger_LogLevel_Deubg,
    VarelaCommon_Logger_LogLevel_Info,
    VarelaCommon_Logger_LogLevel_Warn,
    VarelaCommon_Logger_LogLevel_Error,
};

struct VarelaCommon_Logger_LogEvent {
    struct tm * time;
    const char * file;
    int line;
    enum VarelaCommon_Logger_LogLevel level;
    const char * fmt;
    va_list fmt_varargs;
    void * sink;
};

struct VarelaCommon_Logger_Logger {};

// }}}

// varela-common :: models
// {{{

struct VarelaCommon_Models_Id {
    const char * text;
};

struct VarelaCommon_Models_Entity {
    const char * text;
};

struct VarelaCommon_Models_ExistingEntity {
    struct VarelaCommon_Models_Id id;
    struct VarelaCommon_Models_Entity entity;
};


enum VarelaCommon_Models_Rating {
    VarelaCommon_Models_Rating_Explicit,
    VarelaCommon_Models_Rating_Mature,
    VarelaCommon_Models_Rating_Teen,
    VarelaCommon_Models_Rating_General,
    VarelaCommon_Models_Rating_NotRated,
    VarelaCommon_Models_Rating_Unknown,
};


struct VarelaCommon_Models_Node {
    const char * name;
    const char * key;
    const char * host;
    u16 port;
};

typedef VEC(struct VarelaCommon_Models_Node) VarelaCommon_Models_VecNode;

struct VarelaCommon_Models_Chapter {
    const char * title;
    struct VarelaCommon_Range content;
    Option_Str summary;
    Option_VarelaCommon_Range start_notes;
    Option_VarelaCommon_Range end_notes;
};

typedef VEC(struct VarelaCommon_Models_Chapter) VarelaCommon_Models_VecChapter;


struct VarelaCommon_Models_StoryBase {
    enum VarelaCommon_Models_Site {
        VarelaCommon_Models_Site_ArchiveOfOurOwn,
        VarelaCommon_Models_Site_Unknown,
    } site;
    struct VarelaCommon_Models_StoryInfo {
        enum VarelaCommon_Models_FileKind {
            VarelaCommon_Models_FileKind_Epub,
            VarelaCommon_Models_FileKind_Html,
        } kind;
        u64 file_hash;
        const char * file_name;
        const char * title;
        const char * summary;
        const char * created;
        const char * updated;
    } info;
    VarelaCommon_Models_VecChapter chapters;
};

struct VarelaCommon_Models_Story {
    struct VarelaCommon_Models_StoryBase base;
    struct VarelaCommon_Models_StoryMeta {
        enum VarelaCommon_Models_Rating rating;
    } meta;
};

struct VarelaCommon_Models_ResolvedStory {
    struct VarelaCommon_Models_StoryBase base;
    struct VarelaCommon_Models_ResolvedStoryMeta {
        enum VarelaCommon_Models_Rating rating;
    } meta;
};


struct VarelaCommon_Models_Config {
    enum VarelaCommon_Models_Version {
        VarelaCommon_Models_Version_1,
    } version;
    struct VarelaCommon_Models_Settings {
        enum VarelaCommon_Models_Theme {
            VarelaCommon_Models_Theme_Light,
            VarelaCommon_Models_Theme_Dark,
        } theme;
        const char * data_path;
        const char * temp_path;
        const char * sync_key;
        VarelaCommon_Models_VecNode nodes;
    } settings;
    struct VarelaCommon_Models_Index {} index;
};

// }}}

// }}}

// varela-format-ao3
// {{{
// }}}

// varela-command-config
// {{{
// }}}

// varela-command-index
// {{{
// }}}

// varela-command-serve
// {{{
// }}}

// }}}
