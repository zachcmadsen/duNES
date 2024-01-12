#pragma once

#include <memory>

#include "Nes_Apu.h"

std::unique_ptr<Nes_Apu> nes_apu_new();
