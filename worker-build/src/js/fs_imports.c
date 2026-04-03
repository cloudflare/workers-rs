/**
 * Override the weak standalone stubs from Emscripten's libstandalonewasm.
 *
 * When cargo builds for wasm32-unknown-emscripten, emcc infers
 * STANDALONE_WASM=1 and links libstandalonewasm whose weak stubs resolve
 * FS syscalls to -EPERM inside the WASM binary.  The JS-side MEMFS from
 * `emcc --post-link` can then never supply the real implementations.
 *
 * We provide strong (non-weak) wrappers that forward to the real
 * JS-provided imports.  The __em_js__* trick creates a JS library
 * function that emcc's post-linker will provide.  But since we are in
 * STANDALONE_WASM mode, the simplest approach is to force these functions
 * to remain as wasm imports by re-exporting them via a stub that the
 * Rust side references.
 */
#include <stdarg.h>
#include <stdint.h>

/* ---- JS-backed imports ---- */
__attribute__((import_module("env"), import_name("__syscall_openat")))
int __imported_syscall_openat(int dirfd, intptr_t path, int flags, int mode);

__attribute__((import_module("env"), import_name("__syscall_stat64")))
int __imported_syscall_stat64(intptr_t path, intptr_t buf);

__attribute__((import_module("env"), import_name("__syscall_lstat64")))
int __imported_syscall_lstat64(intptr_t path, intptr_t buf);

__attribute__((import_module("env"), import_name("__syscall_fstat64")))
int __imported_syscall_fstat64(int fd, intptr_t buf);

__attribute__((import_module("env"), import_name("__syscall_newfstatat")))
int __imported_syscall_newfstatat(int dirfd, intptr_t path, intptr_t buf, int flags);

__attribute__((import_module("env"), import_name("__syscall_mkdirat")))
int __imported_syscall_mkdirat(int dirfd, intptr_t path, int mode);

__attribute__((import_module("env"), import_name("__syscall_unlinkat")))
int __imported_syscall_unlinkat(int dirfd, intptr_t path, int flags);

__attribute__((import_module("env"), import_name("__syscall_ioctl")))
int __imported_syscall_ioctl(int fd, int op);

__attribute__((import_module("env"), import_name("__syscall_fcntl64")))
int __imported_syscall_fcntl64(int fd, int cmd);

/* ---- Strong overrides (replace the weak stubs in standalone.c) ---- */

int __syscall_openat(int dirfd, intptr_t path, int flags, ...) {
    int mode = 0;
    if (flags & 0100) {      /* O_CREAT */
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, int);
        va_end(ap);
    }
    return __imported_syscall_openat(dirfd, path, flags, mode);
}

int __syscall_stat64(intptr_t path, intptr_t buf) {
    return __imported_syscall_stat64(path, buf);
}

int __syscall_lstat64(intptr_t path, intptr_t buf) {
    return __imported_syscall_lstat64(path, buf);
}

int __syscall_fstat64(int fd, intptr_t buf) {
    return __imported_syscall_fstat64(fd, buf);
}

int __syscall_newfstatat(int dirfd, intptr_t path, intptr_t buf, int flags) {
    return __imported_syscall_newfstatat(dirfd, path, buf, flags);
}

int __syscall_mkdirat(int dirfd, intptr_t path, int mode) {
    return __imported_syscall_mkdirat(dirfd, path, mode);
}

int __syscall_unlinkat(int dirfd, intptr_t path, int flags) {
    return __imported_syscall_unlinkat(dirfd, path, flags);
}

int __syscall_ioctl(int fd, int op, ...) {
    return __imported_syscall_ioctl(fd, op);
}

int __syscall_fcntl64(int fd, int cmd, ...) {
    return __imported_syscall_fcntl64(fd, cmd);
}

/* Reference this from Rust to ensure the linker pulls in this object. */
int __fs_imports_anchor(void) { return 0; }
