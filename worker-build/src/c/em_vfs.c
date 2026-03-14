/*
 * Emscripten VFS syscall overrides.
 *
 * Emscripten standalone mode (--no-entry) links **weak** stubs from
 * standalone.c that return -EPERM / -ENOSYS for filesystem syscalls.
 * These stubs prevent the syscalls from becoming wasm imports.
 *
 * This file provides **strong** definitions that force the syscalls to
 * appear as wasm imports using the standard emscripten names, so that
 * emcc --post-link automatically provides MEMFS-backed JS implementations.
 *
 * Compiled by worker-build with `emcc -c` before `cargo build`, and
 * passed as `-Clink-arg=<path.o>` so it ends up on the emcc linker
 * command line.  Object files are always linked (unlike archive
 * members), and strong symbols unconditionally override the weak ones.
 */

__attribute__((import_module("env"), import_name("__syscall_openat")))
int __imported_openat(int dirfd, int path, int flags, int mode);

__attribute__((import_module("env"), import_name("__syscall_fstat64")))
int __imported_fstat64(int fd, int buf);

__attribute__((import_module("env"), import_name("__syscall_stat64")))
int __imported_stat64(int path, int buf);

/* Strong definitions override the weak stubs in standalone.c */
int __syscall_openat(int dirfd, int path, int flags, int mode) {
    return __imported_openat(dirfd, path, flags, mode);
}

int __syscall_fstat64(int fd, int buf) {
    return __imported_fstat64(fd, buf);
}

int __syscall_stat64(int path, int buf) {
    return __imported_stat64(path, buf);
}
