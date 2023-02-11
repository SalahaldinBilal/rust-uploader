# Rust file uploader

A simple API built with rust that is used to upload/delete uploaded files from S3 or S3 compatible storage.

## Routes

### - `/upload` POST

The file upload request.\
Body must be of type `multipart/form-data`.\
Must have have a header called `Key` and its value must be equal to the `KEY` env variable.

On success
```JSON
{
  "success": true,
  "data": {
    "url": "the_file_url",
    "deletion": "the_deletion_url?token_str=jwt_token"
  }
}
```

On failure
```JSON
{
  "success": false
}
```

### - `/delete` GET

The file deletion request.\
Must include query parameter `token_str`.

Returns 
```JSON
{
  "success": "will be a boolean",
  "message": "error message or success message"
}
```

### This is my first ever rust project, so might rough around the edges in some places.