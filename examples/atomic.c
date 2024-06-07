#include <stdint.h>

int32_t __atomic_fetch_add_4(volatile int32_t* ptr, int32_t val, int memorder) {
    int32_t old = *ptr;
    *ptr += val;
    return old;
}

uint8_t __atomic_load_1(volatile uint8_t* ptr, int memorder) {
    return *ptr;
}

void __atomic_store_1(volatile uint8_t* ptr, uint8_t val, int memorder) {
    *ptr = val;
}

int32_t __atomic_fetch_sub_4(volatile int32_t* ptr, int32_t val, int memorder) {
    int32_t old = *ptr;
    *ptr -= val;
    return old;
}

int32_t __atomic_load_8(volatile int64_t* ptr, int memorder) {
    return *ptr;
}

int32_t __atomic_load_4(volatile int32_t* ptr, int memorder) {
    return *ptr;
}

void __atomic_store_4(volatile int32_t* ptr, int32_t val, int memorder) {
    *ptr = val;
}

void __atomic_store_8(volatile int64_t* ptr, int64_t val, int memorder) {
    *ptr = val;
}

uint8_t __atomic_exchange_1(volatile uint8_t* ptr, uint8_t val, int memorder) {
    uint8_t old = *ptr;
    *ptr = val;
    return old;
}

int __atomic_compare_exchange_4(volatile int32_t* ptr, int32_t* expected, int32_t desired, int success_memorder, int failure_memorder) {
    if (*ptr == *expected) {
        *ptr = desired;
        return 1;
    } else {
        *expected = *ptr;
        return 0;
    }
}

int __atomic_compare_exchange_8(volatile int64_t* ptr, int64_t* expected, int64_t desired, int success_memorder, int failure_memorder) {
    if (*ptr == *expected) {
        *ptr = desired;
        return 1;
    } else {
        *expected = *ptr;
        return 0;
    }
}
