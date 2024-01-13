#include <memory>

#include "Blip_Buffer.h"
#include "Nes_Apu.h"

std::unique_ptr<Blip_Buffer> blip_buffer_new()
{
    return std::make_unique<Blip_Buffer>();
}

std::unique_ptr<Nes_Apu> nes_apu_new()
{
    return std::make_unique<Nes_Apu>();
}
