#include <memory>

#include "shim.h"

std::unique_ptr<Nes_Apu> nes_apu_new()
{
    return std::make_unique<Nes_Apu>();
}
