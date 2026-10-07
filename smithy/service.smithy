$version: "2"
namespace example.service

use crucible.protocols#json
use crucible.errors#ValidationError
use crucible.errors#InternalError

@json
service MyService {
    version: "1"
    operations: []
    errors: [ValidationError, InternalError]
}
