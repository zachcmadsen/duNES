#pragma once

#include <memory>
#include <system_error>

static_assert(sizeof(std::error_condition) == 16, "");
static_assert(alignof(std::error_condition) == alignof(void*), "");

std::unique_ptr<Blip_Buffer> blip_buffer_new();

std::unique_ptr<Nes_Apu> nes_apu_new();
