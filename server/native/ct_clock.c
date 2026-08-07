#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif

#include "ct_clock.h"

#if defined(_WIN32)
#include <windows.h>

static uint64_t filetime_ticks(FILETIME value) {
    ULARGE_INTEGER integer;
    integer.LowPart = value.dwLowDateTime;
    integer.HighPart = value.dwHighDateTime;
    return integer.QuadPart;
}

int ct_thread_cpu_time_ns(uint64_t* nanoseconds) {
    FILETIME creation;
    FILETIME exit_time;
    FILETIME kernel;
    FILETIME user;
    if(nanoseconds == NULL) {
        return ERROR_INVALID_PARAMETER;
    }
    if(!GetThreadTimes(GetCurrentThread(), &creation, &exit_time, &kernel, &user)) {
        return (int)GetLastError();
    }
    *nanoseconds = (filetime_ticks(kernel) + filetime_ticks(user)) * 100U;
    return 0;
}
#else
#include <errno.h>
#include <time.h>

int ct_thread_cpu_time_ns(uint64_t* nanoseconds) {
    struct timespec value;
    if(nanoseconds == NULL) {
        return EINVAL;
    }
    if(clock_gettime(CLOCK_THREAD_CPUTIME_ID, &value) != 0) {
        return errno;
    }
    *nanoseconds = (uint64_t)value.tv_sec * UINT64_C(1000000000) +
                   (uint64_t)value.tv_nsec;
    return 0;
}
#endif
