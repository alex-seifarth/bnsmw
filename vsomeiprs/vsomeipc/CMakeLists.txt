cmake_minimum_required(VERSION 3.16)

project(vsomeipc
        VERSION 0.1
        LANGUAGES CXX)

cmake_policy(SET CMP0144 NEW)
cmake_policy(SET CMP0074 NEW)

include("cmake/util.cmake")

set(LIB_LOCATIONS)

# Load local.cmake files from one or two folders higher for customization
set(LOCAL_CMAKE_PATHS
    "${CMAKE_CURRENT_LIST_DIR}/.."
    "${CMAKE_CURRENT_LIST_DIR}/../.."
)

foreach(lcp ${LOCAL_CMAKE_PATHS})
    get_filename_component(lcpa "${lcp}" REALPATH)
    if(EXISTS "${lcpa}/local.cmake")
        message(STATUS "Loading local cmake file: ${lcpa}/local.cmake")
        include("${lcpa}/local.cmake")
    endif()
endforeach()

# find dependencies
if(NOT DEFINED vsomeip_VERSION)
    set(vsomeip_VERSION "3.5")
endif()
find_package(vsomeip3 ${vsomeip_VERSION} REQUIRED CONFIG)
get_target_property(VSOMEIP3_LOCATION vsomeip3 LOCATION)

# logging of configuration
message(STATUS " -- Configuration vsomeipc --")
message(STATUS "  - VSOMEIP3_ROOT:    ${vsomeip3_ROOT}")
message(STATUS "  - lib vsomeip:      ${VSOMEIP3_LOCATION} ${VSOMEIP3} ${vsomeip3_FIND_VERSION}")

# vsomeipc library
add_library(vsomeipc STATIC vsomeipc.cpp application.cpp)

target_compile_definitions(vsomeipc PRIVATE CXX_BUILD)
target_link_libraries(vsomeipc PUBLIC vsomeip3)
add_location_entry(LIB_LOCATIONS vsomeip3)

install(TARGETS vsomeipc)

# This file is required by rustc to link to the dynamic libs
set(LIB_LOCATIONS_FILE "${CMAKE_BINARY_DIR}/lib-locations.txt")
string (REPLACE ";" "\n" LIB_LOCATIONS "${LIB_LOCATIONS}")
file(WRITE "${LIB_LOCATIONS_FILE}" "${LIB_LOCATIONS}" )

install(FILES "${LIB_LOCATIONS_FILE}" TYPE LIB)