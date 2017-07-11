/*
 * This file is part of the UnityPack rust package.
 * (c) Istvan Fehervari <gooksl@gmail.com>
 *
 * All rights reserved 2017
 */
#ifndef UNITYPACK_H
#define UNITYPACK_H

/* Asset bundle structure */
struct unitypack_assetbundle;
struct unitypack_asset;
struct unitypack_objectinfo;

struct unitypack_object_array {
    struct unitypack_objectinfo* array;
    size_t length;
};

/* Loads and returns an assetbundle from the give path. Unity assetbundles are usually with .unity3d extension. */
extern const struct unitypack_assetbundle* unitypack_load_assetbundle_from_file(const char* filepath);

/* Destroys a previously loaded assetbundle */
extern void unitypack_destroy_assetbundle(const struct unitypack_assetbundle*);

/* Returns the number of assets inside the given bundle */
extern uint32_t unitypack_get_num_assets(const struct unitypack_assetbundle*);

/* Returns the asset at the given index */
extern struct unitypack_asset* unitypack_get_asset(const struct unitypack_assetbundle*, uint32_t i);

/* Returns the name of the asset. The returned pointer must be freed with unitypack_free_rust_string() */
extern const char* unitypack_get_asset_name(struct unitypack_asset*);

/* Frees a C string created by the unitypack library */
extern void unitypack_free_rust_string(const char*);

extern uint32_t unitypack_get_num_objects(const struct unitypack_asset*, const struct unitypack_assetbundle*);

extern struct unitypack_object_array unitypack_get_objects_with_type(const struct unitypack_asset* asset, const struct unitypack_assetbundle* bundle, const char* object_type);

void unitypack_free_object_array(struct unitypack_object_array* object_array) {
    if (object_array->length != 0) {
        free(object_array->array);
        object_array->length = 0;
    }
};

#endif /* UNITYPACK_H */
