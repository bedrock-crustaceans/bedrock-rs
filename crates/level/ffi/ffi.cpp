#include "ffi.h"

#include <iostream>
#include <memory>
#include <cstring>

#include <leveldb/cache.h>
#include <leveldb/db.h>
#include <leveldb/decompress_allocator.h>
#include <leveldb/env.h>
#include <leveldb/filter_policy.h>
#include <leveldb/options.h>
#include <leveldb/status.h>
#include <leveldb/zlib_compressor.h>
#include <leveldb/write_batch.h>

enum DbStatus translate_status(const leveldb::Status& status) noexcept {
    return static_cast<DbStatus>(status.code());
}

void copy_string(FfiResult* result, const std::string& error) {
    const char* src = error.c_str();
    size_t src_size = error.size() + 1; // Make space for null.

    result->size = static_cast<int>(src_size);
    result->data = new char[src_size];

    memcpy(result->data, src, src_size);
}

class EmptyLogger : public leveldb::Logger {
public:
    void Logv(const char* fmt, va_list args) override {}
};

struct Database {
    leveldb::Options options = leveldb::Options();
    leveldb::WriteOptions write_options{};
    leveldb::ReadOptions read_options{};
    leveldb::DB* db = nullptr;

    ~Database() noexcept {
        delete this->db;

        delete this->read_options.decompress_allocator;

        delete this->options.compressors[1];
        delete this->options.compressors[0];
        delete this->options.info_log;
        delete this->options.block_cache;
        delete this->options.filter_policy;
    }
};

FfiResult db_open(const char* path) {
    FfiResult result{};

    std::unique_ptr<Database> database = std::make_unique<Database>();

    database->options.filter_policy = leveldb::NewBloomFilterPolicy(10);
    database->options.block_cache = leveldb::NewLRUCache(40 * 1024 * 1024);
    database->options.info_log = new EmptyLogger();
    database->options.compressors[0] = new leveldb::ZlibCompressorRaw();
    database->options.compressors[1] = new leveldb::ZlibCompressor();
    
    database->read_options.decompress_allocator = new leveldb::DecompressAllocator();

    leveldb::Status status = leveldb::DB::Open(database->options, path, &database->db);

    result.status = translate_status(status);

    if(status.ok()) {
        result.size = sizeof(Database);
        result.data = database.release();
    } else {    
        std::string error = status.ToString();
        copy_string(&result, error);
    }

    return result;
}

void db_close(void* db_ptr) {
    Database* database = reinterpret_cast<Database*>(db_ptr);
    delete database;
}

FfiResult db_get(void* db_ptr, const char* key, int key_size) {
    FfiResult result{};

    Database* db = reinterpret_cast<Database*>(db_ptr);

    std::string value;
    leveldb::Status status = db->db->Get(db->read_options, leveldb::Slice(key, key_size), &value);

    result.status = translate_status(status);
    if(status.ok()) {
        result.size = static_cast<int>(value.size());
        result.data = new char[value.size()];

        // TODO: This memcpy can probably be removed.
        memcpy(result.data, value.data(), value.size());
    } else {
        std::string error = status.ToString();
        copy_string(&result, error);
    }

    return result;
}

FfiResult db_put(
    void* db_ptr,
    const char* key, int key_size,
    const char* val, int val_size
) {
    Database* db = reinterpret_cast<Database*>(db_ptr);
    FfiResult result{};

    leveldb::Slice key_slice(key, key_size);
    leveldb::Slice val_slice(val, val_size);

    leveldb::Status status = db->db->Put(db->write_options, key_slice, val_slice);
    result.status = translate_status(status);

    if(status.ok()) {
        result.data = nullptr;
        result.size = 0;
    } else {
        std::string error = status.ToString();
        copy_string(&result, error);
    }

    return result;
}

FfiResult db_remove(void* db_ptr, const char* key, int key_size) {
    Database* db = reinterpret_cast<Database*>(db_ptr);
    FfiResult result{};

    leveldb::Slice key_slice(key, key_size);

    leveldb::Status status = db->db->Delete(db->write_options, key_slice);

    result.status = translate_status(status);
    if(status.ok()) {
        result.data = nullptr;
        result.size = 0;
    } else {
        std::string error = status.ToString();
        copy_string(&result, error);
    }

    return result;
}

void buffer_destroy(char* array) {
    delete[] array;
}