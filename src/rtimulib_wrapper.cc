#include <cstdio>

#include "RTIMULib.h"

struct WrapperContext {
    RTIMUSettings* p_settings;
    RTIMU* p_imu;
    uint32_t magic;
};

struct Orientation {
    double x;
    double y;
    double z;
};

extern "C" {
    WrapperContext* rtimulib_wrapper_create(void);
    void rtimulib_wrapper_destroy(WrapperContext* p_context);
    int rtimulib_wrapper_imu_read(WrapperContext* p_context);
    int rtimulib_wrapper_get_imu_data(WrapperContext* p_context, Orientation* p_output);
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

int rtimulib_wrapper_imu_read(WrapperContext* p_context) {
    return p_context->p_imu->IMURead();
}

int rtimulib_wrapper_get_imu_data(WrapperContext* p_context, Orientation* p_output) {
    RTIMU_DATA imuData = p_context->p_imu->getIMUData();
    if (imuData.fusionPoseValid && p_output) {
        p_output->x = imuData.fusionPose.x();
        p_output->y = imuData.fusionPose.y();
        p_output->z = imuData.fusionPose.z();
    };
    return imuData.fusionPoseValid;
}
