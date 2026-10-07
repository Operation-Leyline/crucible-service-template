$version: "2"
namespace example.service

use crucible.protocols#json
use crucible.errors#ValidationException
use crucible.errors#InternalServerException
use crucible.errors#UnknownOperationException

@json
service MyService {
    version: "1"
    operations: []
    errors: [ValidationException, InternalServerException, UnknownOperationException]
}
