#ifdef __cplusplus
extern "C" {
#endif __cplusplus

enum FfiStatus {
    Success,
    NotFound,
    Corrupted,
    NotSupported,
    InvalidArgument,
    IoError,
    AllocationFailed,
    Exception
};

struct FfiResult {
    enum FfiStatus status;
    int size;
    void* data;
};

struct FfiResult bedrockrs_db_open(const char* path);
void bedrockrs_db_close(void* db);
struct FfiResult bedrockrs_db_get(void* db, const char* key, int key_size);
struct FfiResult bedrockrs_db_put(void* db, const char* key, int key_size, const char* val, int val_size);
struct FfiResult bedrockrs_db_remove(void* db, const char* key, int key_size);
void bedrockrs_buffer_destroy(char* array);
struct FfiResult bedrockrs_iter_new(void* db);
void bedrockrs_iter_destroy(void* iter);
struct FfiResult bedrockrs_iter_key(const void* iter);
struct FfiResult bedrockrs_iter_value(const void* iter);
bool bedrockrs_iter_valid(const void* iter);
void bedrockrs_iter_next(void* iter);

// void* batch_new();

// void batch_delete(void* batch, const char* key, int key_size);

// void batch_put(void* batch, const char* key, int key_size, const char* val, int val_size);

// void batch_clear(void* batch);

// void batch_destroy(void* batch);

// struct FfiResult batch_execute(void* db, void* batch);

#ifdef __cplusplus
}
#endif __cplusplus