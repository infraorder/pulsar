const vec2 blurdir = vec2( 1.0, 0.0 );

const float blurdist_px = 8.0;
const int NUM_SAMPLES = 6;

const float THRESHOLD = 0.1;
const float MULT = 2.0;

vec3 srgb2lin(vec3 c) { return c*c; }
vec3 lin2srgb(vec3 c) { return sqrt(c); }

//note: uniform pdf rand [0;1[
float hash12n(vec2 p)
{
	p  = fract(p * vec2(5.3987, 5.4421));
    p += dot(p.yx, p.xy + vec2(21.5351, 14.3137));
	return fract(p.x * p.y * 95.4307);
}

vec4 pattern( vec2 p )
{
    float aspect = iResolution.x/iResolution.y;
    float p0 = step(abs(p.x-0.125), 0.01) * step(abs(p.y-0.27), 0.01);
    float p1 = step( length( p-vec2(0.125, 0.45) ), 0.025 );
    
    float p2_0 = step( length( p-vec2(0.08, 0.14) ), 0.0125 );
    float p2_1 = step( length( p-vec2(0.16, 0.125) ), 0.0125 );
    float p2_2 = step( length( p-vec2(0.1, 0.07) ), 0.0125 );
    float p2 = max(p2_0, max(p2_1,p2_2));
    
    return vec4( max( p0, max(p1,p2) ) );
}

vec3 sampletex( vec2 uv )
{
    float t = fract( 0.1*iTime );
    if ( t < 1.0/3.0)
    	return srgb2lin( texture( iChannel0, uv, -10.0 ).rgb );
    else if ( t < 2.0/3.0 )
        return srgb2lin( texture( iChannel1, uv, -10.0 ).rgb );
    else
	    return srgb2lin( texture( iChannel2, uv, -10.0 ).rgb );    
}

vec3 genimg( vec2 suv, vec2 p )
{
    if ( suv.x < 0.25 )
        return pattern( p ).rgb;
    else
    {
        vec3 smpl = (sampletex(p) - THRESHOLD) / (1.0-THRESHOLD);
        //sumcol += smpl;
        return smpl*smpl; //wtf
    }
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    fragCoord += 25.0 * vec2( cos(iTime), sin(iTime ) );
    
    vec2 suv = fragCoord / iResolution.xy; 
    vec2 uv = fragCoord / iResolution.xx;

    float blurdist = (iMouse.z>0.5) ? 32.0 * iMouse.x/iResolution.x : blurdist_px;
   
    float da = 6.283 / float(NUM_SAMPLES);
    float a = da * hash12n(uv+fract(iTime));

    vec3 sumcol = vec3(0.0);
    for (int i=0;i<NUM_SAMPLES;++i)
    {
        vec2 ofs = vec2( cos(a), sin(a) ) / iResolution.x * blurdist;
        vec2 p = uv + ofs;

 		sumcol += genimg( suv, p );
        
        a += da;
    }
    sumcol /= float(NUM_SAMPLES);
    sumcol = max( sumcol, 0.0 );
    
    fragColor = vec4( lin2srgb( sumcol * MULT ), 1.0 );
}

