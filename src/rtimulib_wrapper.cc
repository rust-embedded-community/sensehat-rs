#include "RTIMULib.h"

struct WrapperContext {
    RTIMUSettings* p_settings;
    RTIMU* p_imu;
};

struct Vector3D {
    double x;
    double y;
    double z;
};

struct AllData {
    uint64_t timestamp;
    int fusionPoseValid;
    Vector3D fusionPose;
    int gyroValid;
    Vector3D gyro;
    int accelValid;
    Vector3D accel;
    int compassValid;
    Vector3D compass;
    int pressureValid;
    double pressure;
    int temperatureValid;
    double temperature;
    int humidityValid;
    double humidity;
};

extern "C" {
    WrapperContext* rtimulib_wrapper_create(void);
    void rtimulib_wrapper_destroy(WrapperContext* p_context);
    void rtimulib_set_sensors(WrapperContext* p_context, int gyro, int accel, int compass);
    int rtimulib_wrapper_imu_read(WrapperContext* p_context);
    int rtimulib_wrapper_get_imu_data(WrapperContext* p_context, AllData* p_output);
}

WrapperContext* rtimulib_wrapper_create(void) {
    WrapperContext* p_context = new WrapperContext;
    // TODO: Should be ~/.config/sense_hat/RTIMULib
    p_context->p_settings = new RTIMUSettings("RTIMULib");
    p_context->p_imu = RTIMU::createIMU(p_context->p_settings);
    p_context->p_imu->IMUInit();
    p_context->p_imu->setSlerpPower(0.02);
    rtimulib_set_sensors(p_context, 1, 1, 1);
    return p_context;
}

void rtimulib_wrapper_destroy(WrapperContext* p_context) {
    // The settings object must outlive the IMU object
    delete p_context->p_imu;
    delete p_context->p_settings;
    delete p_context;
}

void rtimulib_set_sensors(WrapperContext* p_context, int gyro, int accel, int compass) {
    p_context->p_imu->setGyroEnable(gyro);
    p_context->p_imu->setAccelEnable(accel);
    p_context->p_imu->setCompassEnable(compass);
}

int rtimulib_wrapper_imu_read(WrapperContext* p_context) {
    return p_context->p_imu->IMURead();
}

int rtimulib_wrapper_get_imu_data(WrapperContext* p_context, AllData* p_output) {
    RTIMU_DATA imuData = p_context->p_imu->getIMUData();
    p_output->timestamp = imuData.timestamp;
    p_output->fusionPoseValid = imuData.fusionPoseValid;
    if (p_output->fusionPoseValid) {
        p_output->fusionPose.x = imuData.fusionPose.x();
        p_output->fusionPose.y = imuData.fusionPose.y();
        p_output->fusionPose.z = imuData.fusionPose.z();
    }
    p_output->gyroValid = imuData.gyroValid;
    if (p_output->gyroValid) {
        p_output->gyro.x = imuData.gyro.x();
        p_output->gyro.y = imuData.gyro.y();
        p_output->gyro.z = imuData.gyro.z();
    }
    p_output->accelValid = imuData.accelValid;
    if (p_output->accelValid) {
        p_output->accel.x = imuData.accel.x();
        p_output->accel.y = imuData.accel.y();
        p_output->accel.z = imuData.accel.z();
    }
    p_output->compassValid = imuData.compassValid;
    if (p_output->compassValid) {
        p_output->compass.x = imuData.compass.x();
        p_output->compass.y = imuData.compass.y();
        p_output->compass.z = imuData.compass.z();
    }
    p_output->pressureValid = imuData.pressureValid;
    if (p_output->pressureValid) {
        p_output->pressure = imuData.pressure;
    }
    p_output->temperatureValid = imuData.temperatureValid;
    if (p_output->temperatureValid) {
        p_output->temperature = imuData.temperature;
    }
    p_output->humidityValid = imuData.humidityValid;
    if (p_output->humidityValid) {
        p_output->humidity = imuData.humidity;
    }
    return 1;
}
