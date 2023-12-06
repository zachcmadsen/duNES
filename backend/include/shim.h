#pragma once

#include <memory>

#include "Blip_Buffer.h"
#include "Nes_Apu.h"

std::unique_ptr<Blip_Buffer> new_blip_buffer();

std::unique_ptr<Nes_Apu> new_nes_apu();
