#include <memory>

#include "shim.h"

std::unique_ptr<Blip_Buffer> new_blip_buffer()
{
    return std::make_unique<Blip_Buffer>();
}

std::unique_ptr<Nes_Apu> new_nes_apu()
{
    return std::make_unique<Nes_Apu>();
}
