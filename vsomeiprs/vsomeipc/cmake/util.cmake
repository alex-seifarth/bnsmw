function(add_location_entry variable target)
    get_target_property(location ${target} LOCATION)
    list(APPEND ${variable} "${target}:${location}")
    set(${variable} ${${variable}} PARENT_SCOPE)
endfunction()
