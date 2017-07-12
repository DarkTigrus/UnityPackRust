/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>
#include <unitypack.h>

int main() {
    const char* filepath = "/Applications/Hearthstone/Data/OSX/cards0.unity3d";
    uint32_t i, j;

    const struct unitypack_assetbundle* assetbundle = unitypack_load_assetbundle_from_file(filepath);
    printf("Successfully loaded assetbundle from %s\n",filepath);

    uint32_t num_assets = unitypack_get_num_assets(assetbundle);
    printf("There are %d asset(s) in the bundle\n",num_assets);

    for (i=0; i < num_assets; i++) {
        struct unitypack_asset* asset = unitypack_get_asset(assetbundle, i);
        const char* asset_name = unitypack_get_asset_name(asset);
        printf("Asset %d: %s\n",i,asset_name);
        unitypack_free_rust_string(asset_name);

        uint32_t num_objects = unitypack_get_num_objects(asset, assetbundle);
        printf("There are %d objects in the asset\n",num_objects);

        struct unitypack_object_array object_array = unitypack_get_objects_with_type(asset, assetbundle, "GameObject");

        for (j=0; j < object_array.length; j++) {
            const char* object_type = unitypack_get_object_type(object_array.array[j], asset, assetbundle);
            printf("%s\n", object_type);
            unitypack_free_rust_string(object_type);
        }
        

        unitypack_free_object_array(&object_array);
    }
    
    unitypack_destroy_assetbundle(assetbundle);
    return 0;
}