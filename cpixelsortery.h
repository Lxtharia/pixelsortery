#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

void hilbert_glitch(uint32_t *data, uint32_t width, uint32_t height);

void swaylock_effect(uint32_t data[], int width, int height) {
    // data = [a b g r][a b g r]....
    hilbert_glitch(data, width, height);
}
