#include <stdlib.h>
#include <stdint.h>
#include <math.h>


uint32_t times_pi_rounded(uint32_t n) {
    return (uint32_t) round(4*atan(1)*n);
}

uint8_t flip_coin() {
    return (rand()%2);
}


