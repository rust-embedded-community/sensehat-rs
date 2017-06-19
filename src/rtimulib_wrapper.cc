#include <cstdio>

#include "RTIMULib.h"

struct WrapperContext {
    RTIMUSettings* p_settings;
    RTIMU* p_imu;
    uint32_t magic;
};

extern "C" {
    WrapperContext* rtimulib_wrapper_create(void);
    void rtimulib_wrapper_destroy(WrapperContext* p_context);
}

WrapperContext* rtimulib_wrapper_create(void) {
    WrapperContext* p_context = new WrapperContext;
    p_context->magic = 0xDEADBEEF;
    printf("Magic at create is %08x\n", p_context->magic);
    p_context->p_settings = new RTIMUSettings("RTIMULib");
    p_context->p_imu = RTIMU::createIMU(p_context->p_settings);
    p_context->p_imu->IMUInit();
    p_context->p_imu->setSlerpPower(0.02);
    p_context->p_imu->setGyroEnable(true);
    p_context->p_imu->setAccelEnable(true);
    p_context->p_imu->setCompassEnable(true);
    return p_context;
}

void rtimulib_wrapper_destroy(WrapperContext* p_context) {
    printf("Magic at destroy is %08x\n", p_context->magic);
    delete p_context->p_settings;
    delete p_context->p_imu;
    delete p_context;
}
